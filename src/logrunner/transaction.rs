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
        let cpu = {
            let memory = self.mems.to_memory();
            let state = &self.state;
            let mut cpu: Processor<ByteMap> = RestorableState { state, memory }.into();
            cpu.step(&matchers);
            cpu
        };

        let mut fail = if !self.after.validate(&cpu, Some(self.state.clone())) {
            error!("cpu state transaction fail");
            true
        } else {
            false
        };

        if let Some(ref store) = self.store {
            if !store.validate(cpu.mmu()) {
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
