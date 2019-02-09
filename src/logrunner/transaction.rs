use super::bincode::TupleReader;
use super::{Insn, LogTuple, MemoryTrace, RestorableState, State, ToMemory};
use crate::{build_matchers, matcher::Matcher, memory::ByteMap, Processor};
use std::io;

pub fn single() -> Result<(), io::Error> {
    let matchers = build_matchers::<ByteMap>();
    let mut reader = io::BufReader::new(io::stdin());
    let t: Transaction = bincode::deserialize_from(&mut reader).expect("read transaction");
    t.validate(&matchers);
    Ok(())
}

pub fn validatestream() -> Result<(), io::Error> {
    let matchers = build_matchers::<ByteMap>();
    let mut reader = io::BufReader::new(io::stdin());

    use std::time::SystemTime;
    let mark = SystemTime::now();
    let mut count = 0;

    loop {
        let t: Transaction = match bincode::deserialize_from(&mut reader) {
            Ok(t) => t,
            Err(e) => {
                if let bincode::ErrorKind::Io(ref e) = *e {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        break;
                    }
                }
                error!("Failed to read trans: {}", e);
                panic!("Failed to read trans");
            }
        };
        t.validate(&matchers);
        count += 1;
    }

    let d = SystemTime::now().duration_since(mark).expect("time");
    let in_ms = d.as_secs() * 1000 + d.subsec_nanos() as u64 / 1_000_000;
    let in_sec = (in_ms as f32) / 1000f32;
    let speed = (count as f32) / in_sec;
    println!("Validated {} transactions @ {} per/sec", count, speed);

    Ok(())
}

use flate2::write::GzEncoder;
use flate2::Compression;

struct TransLogFile {
    name: String,
    count: usize,
    idx: usize,
    mask: u32,
    mtch: u32,
    skip: bool,
    f: Option<GzEncoder<io::BufWriter<std::fs::File>>>,
}

impl TransLogFile {
    fn new(mask: u32, mtch: u32, name: &str) -> Self {
        let mut t = TransLogFile {
            name: name.into(),
            count: 0,
            idx: 0,
            mask,
            mtch,
            skip: false,
            f: None,
        };
        t.open_file();
        t
    }
    fn open_file(&mut self) {
        let ord = if self.idx == 0 {
            String::new()
        } else {
            format!(".{}", self.idx)
        };
        self.f = Some(GzEncoder::new(
            io::BufWriter::new(
                std::fs::File::create(format!(
                    "assets/transactions-logs/{}{}.trans.log.gz",
                    self.name, ord
                ))
                .expect("create trans.log"),
            ),
            Compression::default(),
        ));
    }
    fn rotate_file(&mut self) {
        self.f.take();
        self.idx += 1;
        self.count = 0;
        self.open_file();
    }
    fn store(&mut self, t: &Transaction) -> bool {
        if let Some(insn) = &t.insn {
            if insn.bits & self.mask == self.mtch {
                if self.skip {
                    return true;
                }
                bincode::serialize_into(self.f.as_mut().expect("f"), &t).expect("serialize trans");
                self.count += 1;
                if self.count >= 250_000 {
                    self.rotate_file();
                }
                return true;
            }
        }
        false
    }
}

pub fn filter() -> Result<(), io::Error> {
    let stdin = io::stdin();
    let handle = stdin.lock();
    let reader = super::bincode::LogLineReader::new(io::BufReader::new(handle)).to_tuple();

    let mut logs = vec![
        TransLogFile::new(0x7f, 0b000_0011, "load"),
        TransLogFile::new(0x7f, 0b010_0011, "store"),
        TransLogFile::new(0x7f, 0b011_0011, "comp"),
        TransLogFile::new(0x7f, 0b001_0011, "imm"),
        TransLogFile::new(0x7f, 0b001_1011, "immw"),
        TransLogFile::new(0x7f, 0b110_0011, "branch"),
        TransLogFile::new(0x7f, 0b011_1011, "wide"),
        TransLogFile::new(0x7f, 0b111_0011, "system"),
        TransLogFile::new(0x7f, 0b010_1111, "amo"),
    ];

    let mut other = TransLogFile::new(0, 0, "other");

    let mut stored = false;
    for t in TransactionIterator::new(reader)
        .skip(180_000_000)
        .take(10_000_000)
    {
        stored = false;

        for log in &mut logs {
            if log.store(&t) {
                stored = true;
                break;
            }
        }

        if !stored {
            other.store(&t);
        }
    }

    println!("\n\n{}\n", logs[1].count);

    Ok(())
}

pub fn stream() -> Result<(), io::Error> {
    let matchers = build_matchers::<ByteMap>();

    let stdin = io::stdin();
    let handle = stdin.lock();
    let reader = super::bincode::LogLineReader::new(io::BufReader::new(handle)).to_tuple();

    for t in TransactionIterator::new(reader) {
        t.validate(&matchers);
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Transaction {
    pub(crate) state: State,
    pub(crate) insn: Option<Insn>,
    pub(crate) mems: Vec<MemoryTrace>,
    pub(crate) store: Option<MemoryTrace>,
    pub(crate) after: State,
}

impl Transaction {
    pub fn validate(&self, matchers: &[Matcher<ByteMap>]) {
        let mut cpu = {
            let memory = self.mems.to_memory();
            let state = &self.state;
            let mut cpu: Processor<ByteMap> = RestorableState { state, memory }.into();
            cpu.step(matchers.iter());
            cpu
        };

        let mut fail = if !self.after.validate(&cpu, Some(self.state.clone())) {
            error!("cpu state transaction fail");
            true
        } else {
            false
        };

        if let Some(ref store) = self.store {
            if !store.validate(cpu.mmu_mut()) {
                error!("mem store transaction fail");
                fail = true;
            }
        }

        if fail {
            self.save_to("failed.bincode");
            // error!("transaction failed\n{:?}", self);
            if let Some(ref insn) = self.insn {
                error!("Insn: {}", insn.desc);
            } else {
                error!("Insn: None");
            }
            error!("Before {}", self.state);
            error!("After  {}", self.after);
            panic!("transaction failed");
        } else {
            info!("ok");
        }
    }

    fn save_to(&self, filename: &str) {
        use bincode;
        use std::fs::File;
        let mut out = File::create(filename).expect("create save file");
        bincode::serialize_into(&mut out, self).expect("save_to");
    }
}

pub(crate) struct TransactionIterator<I = TupleReader> {
    last_tuple: LogTuple,
    it: I,
}

impl<T: Iterator<Item = LogTuple>> TransactionIterator<T> {
    fn new(mut it: T) -> Self {
        let last_tuple = it.next().expect("no transaction data");
        TransactionIterator { last_tuple, it }
    }
}

impl Default for TransactionIterator {
    fn default() -> Self {
        let mut it = TupleReader::new();
        let last_tuple = it.next().expect("no transaction data");
        TransactionIterator { last_tuple, it }
    }
}

impl<I> Iterator for TransactionIterator<I>
where
    I: Iterator<Item = LogTuple>,
{
    type Item = Transaction;

    fn next(&mut self) -> Option<Transaction> {
        let tuple = if let Some(t) = self.it.next() {
            t
        } else {
            return None;
        };

        let after = tuple.state.clone();
        let this_tuple = self.last_tuple.clone();
        self.last_tuple = tuple.clone();

        // all values of the next run
        // clone state is our after
        let LogTuple {
            line,
            state,
            insn,
            store,
            mems,
        } = this_tuple;
        let _line = line;

        Some(Transaction {
            state,
            insn,
            mems,
            store,
            after,
        })
    }
}
