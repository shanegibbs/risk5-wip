use super::{Insn, MemoryTrace, RestorableState, State, ToMemory};
use crate::matcher::Matcher;
use crate::memory::*;
use crate::Processor;
use std::io;

pub fn validate() -> Result<(), io::Error> {
    let matchers = crate::build_matchers::<ByteMap>();

    use bincode;
    let mut reader = io::BufReader::new(io::stdin());
    let t: Transaction = bincode::deserialize_from(&mut reader).expect("read transaction");
    t.validate(&matchers);
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Transaction {
    pub(crate) state: State,
    pub(crate) insn: Insn,
    pub(crate) mems: Vec<MemoryTrace>,
    pub(crate) store: Option<MemoryTrace>,
    pub(crate) after: State,
}

impl Transaction {
    pub fn validate(&self, matchers: &[Matcher<ByteMap>]) {
        let cpu = {
            let memory = self.mems.to_memory();
            let state = &self.state;
            let mut cpu: Processor<ByteMap> = RestorableState {
                state: &self.state,
                memory,
            }
            .into();
            cpu.step(&matchers);
            cpu
        };

        let mut fail = if !self.after.validate(&cpu) {
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
            error!("transaction failed\n{:?}", self);
            self.save_to("failed.bincode");
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