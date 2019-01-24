use super::bincode::BincodeReader;
use super::{Insn, LogTuple, MemoryTrace, RestorableState, State, ToMemory};
use crate::matcher::Matcher;
use crate::memory::*;
use crate::Processor;
use std::io;

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

// TODO iterator of current and next state
// TODO multithread

impl Transaction {
    fn validate(&self, matchers: &[Matcher<ByteMap>]) {
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

        let mut fail = false;

        if !self.after.validate(&cpu) {
            error!("cpu state transaction fail");
            fail = true;
        }

        if let Some(ref store) = self.store {
            if !store.validate(cpu.mmu()) {
                error!("mem store transaction fail");
                fail = true;
            }
        }

        if fail {
            error!("transaction failed");
            panic!("transaction failed");
        } else {
            info!("ok");
        }
    }
}

struct Transaction {
    state: State,
    insn: Insn,
    mems: Vec<MemoryTrace>,
    store: Option<MemoryTrace>,
    after: State,
}

fn maybe_test_state(
    matchers: &[Matcher<ByteMap>],
    last_state: &Option<State>,
    last_insn: &Option<Insn>,
    last_mems: &[MemoryTrace],
    state: &State,
    last_store: &Option<MemoryTrace>,
) {
    if last_mems.iter().any(|m| m.addr <= 0x10000) {
        return;
    }

    let before = if let Some(t) = last_state { t } else { return };
    let insn = if let Some(t) = last_insn { t } else { return };

    if before.pc < 0x10000 || state.pc < 0x10000 {
        return;
    }

    let transaction = Transaction {
        state: before.to_owned(),
        insn: insn.to_owned(),
        mems: Vec::from(last_mems),
        store: last_store.to_owned(),
        after: state.to_owned(),
    };

    transaction.validate(matchers);
}

#[inline(never)]
fn run_log<I>(logs: I) -> Result<(), io::Error>
where
    I: Iterator<Item = LogTuple>,
{
    let matchers = crate::build_matchers::<ByteMap>();

    let mut dtb_mem = ByteMap::default();
    let dtb = crate::load_dtb();
    crate::write_reset_vec(&mut dtb_mem, 0x8000_0000, &dtb);
    let persistent = dtb_mem.into_data();

    let mem = ByteMap::default().with_persistent(persistent.clone());
    let mut cpu = Processor::new(0x1000, mem);

    let mut last_insn: Option<Insn> = None;
    let mut last_state: Option<State> = None;
    let mut last_store: Option<MemoryTrace> = None;
    let mut last_mems = vec![];

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

        if !cpu.mmu().mem().did_persistent_load() {
            maybe_test_state(
                &matchers,
                &last_state,
                &last_insn,
                &last_mems,
                &state,
                &last_store,
            );
        }

        // trace!("This insn {:?}", insn);

        let mut fail = false;

        // validate current state

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
            if !store.validate(cpu.mmu()) {
                fail = true;
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
                .unwrap_or_else(|| "None".into())
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

        for mem in &mems {
            cpu.mmu_mut().mem_mut().write_b(mem.addr, mem.value as u8);
        }

        cpu.step(&matchers);

        last_insn = insn;
        last_state = Some(state);
        last_store = store;
        last_mems = mems;
    }

    warn!("Retired {} insns", counter);
    Ok(())
}
