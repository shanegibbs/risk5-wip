use super::*;
use crate::memory::*;
use crate::Memory;

pub fn run() -> Result<(), io::Error> {
    super::logger::init().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let reader = BincodeReader::new();

    use std::env;
    if let Ok(val) = env::var("STOP_AT") {
        let stop_at = val.parse::<usize>().expect("parse STOP_AT");
        return run_log(reader.take(stop_at));
    }

    run_log(reader)
}

pub fn convert() -> Result<(), io::Error> {
    use bincode;
    use std::io::BufWriter;

    let mut out = BufWriter::new(io::stdout());

    info!("starting");

    for line in JsonLogTupleIterator::new()? {
        trace!("{:?}", line);
        let bin = line.to_logtuple();
        bincode::serialize_into(&mut out, &bin).map_err(|e| match *e {
            bincode::ErrorKind::Io(e) => e,
            e => io::Error::new(io::ErrorKind::Other, format!("{}", e)),
        })?
    }

    Ok(())
}

struct BincodeReader {
    reader: BufReader<io::Stdin>,
}

impl BincodeReader {
    fn new() -> Self {
        let reader = BufReader::new(io::stdin());
        BincodeReader { reader }
    }
}

impl Iterator for BincodeReader {
    type Item = LogTuple;

    fn next(&mut self) -> Option<LogTuple> {
        use bincode;

        let val = match bincode::deserialize_from(&mut self.reader) {
            Ok(n) => n,
            Err(e) => {
                match *e {
                    bincode::ErrorKind::Io(ref e) => {
                        if e.kind() == io::ErrorKind::UnexpectedEof {
                            return None;
                        }
                    }
                    _ => (),
                }
                error!("Failed to read log tuple: {}", e);
                panic!("Failed to read log tuple");
            }
        };

        Some(val)
    }
}

#[inline(never)]
fn run_log<I>(logs: I) -> Result<(), io::Error>
where
    I: Iterator<Item = LogTuple>,
{
    let matchers = crate::build_matchers::<ByteMap>();

    let mut dtb_mem = ByteMap::new();
    let dtb = crate::load_dtb();
    crate::write_reset_vec(&mut dtb_mem, 0x80000000, &dtb);

    let mem = ByteMap::new().with_persistent(dtb_mem.to_data());
    let mut cpu = Processor::new(0x1000, mem);

    let mut last_insn: Option<Insn> = None;
    let mut last_state: Option<State> = None;
    let mut last_store: Option<MemoryTrace> = None;

    let mut counter = 0;

    for (step, log_tuple) in logs.enumerate() {
        let LogTuple {
            line,
            state,
            insn,
            store,
            mems,
        } = log_tuple;
        counter += 1;

        // trace!("This insn {:?}", insn);

        let mut fail = false;

        // validate current state

        macro_rules! fail_on {
            ($name:expr, $expected:expr, $actual:expr) => {
                if $expected != $actual {
                    error!(
                        "Fail check on {}.\n{}",
                        $name,
                        format_diff($expected, $actual)
                    );
                    fail = true;
                }
            };
        }

        if let Some(ref insn) = &insn {
            if insn.pc != state.pc {
                error!("Insn and state pc do not match");
                fail = true;
            }
        }

        if !state.validate(&cpu) {
            fail = true;
        }

        // check for memory store
        if let Some(store) = last_store {
            trace!("Checking store {}", store);
            if store.addr >= 0x80009000 && store.addr < 0x80009016 {
                trace!("Ignoring write to HTIF: {}", store);
            } else {
                match store.kind.as_str() {
                    "uint8" => {
                        let val = cpu.mmu().read_b(store.addr).expect("mmu read");
                        fail_on!("store", store.value as u8, val);
                    }
                    "uint16" => {
                        let val = cpu.mmu().read_h(store.addr).expect("mmu read");
                        fail_on!("store", store.value as u16, val);
                    }
                    "uint32" => {
                        let val = cpu.mmu().read_w(store.addr).expect("mmu read");
                        fail_on!("store", store.value as u32, val);
                    }
                    "uint64" => {
                        let val = cpu.mmu().read_d(store.addr).expect("mmu read");
                        fail_on!("store", store.value, val);
                    }
                    n => unimplemented!("check store type {}", n),
                }
            }
        }

        if fail {
            error!("debug info - step:     {}", step as i64 - 1);
            error!(
                "debug info - Insn PC:  0x{:x}",
                insn.map(|i| i.pc).unwrap_or(0)
            );
            error!("debug info - State PC: 0x{:x}", state.pc);
            if let Some(last_insn) = last_insn {
                error!("debug info - Insn:     {}", last_insn.desc);
            }
            error!("debug info - Line:     {}", line);
            panic!("Failed checks");
        }

        trace!(
            "State validated OK from: {}",
            last_insn
                .map(|l| format!("{}", l))
                .unwrap_or(format!("None"))
        );

        // reset and begin again

        cpu.mmu_mut().mem_mut().clear();

        let status_line = if let Some(ref insn) = insn {
            format!("--- Begin step {} (json line {}) --- {}", step, line, insn)
        } else {
            format!("--- Begin step {} (json line {}) --- No insn", step, line)
        };

        if step % 10000 == 0 {
            warn!("{}", status_line);
        }
        info!("{}", status_line);

        for mem in mems {
            cpu.mmu_mut().mem_mut().write_b(mem.addr, mem.value as u8);
        }

        cpu.step(&matchers);

        last_insn = insn;
        last_state = Some(state);
        last_store = store;
    }

    warn!("Retired {} insns", counter);
    Ok(())
}
