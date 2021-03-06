use super::bincode;
use super::{Insn, LogTuple, MemoryTrace, State, Transaction};
use crate::matcher::Matchers;
use crate::memory::*;
use crate::Processor;
use std::io;

pub fn run() -> Result<(), io::Error> {
    super::logger::init().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let reader = bincode::LogLineReader::new(io::BufReader::new(io::stdin())).to_tuple();

    use std::env;
    if let Ok(val) = env::var("STOP_AT") {
        let stop_at = val.parse::<usize>().expect("parse STOP_AT");
        return run_log(reader.take(stop_at));
    }

    run_log(reader)
}

// TODO multithread

fn test_state(
    matchers: &mut Matchers<ByteMap>,
    last_state: &Option<State>,
    last_insn: &Option<Insn>,
    last_mems: &[MemoryTrace],
    state: &State,
    last_store: &Option<MemoryTrace>,
) {
    let before = if let Some(t) = last_state { t } else { return };

    let transaction = Transaction {
        state: before.to_owned(),
        insn: last_insn.to_owned(),
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
    let matchers = &mut crate::build_matchers::<ByteMap>();

    let mut dtb_mem = ByteMap::default();
    let dtb = crate::load_dtb();
    crate::write_reset_vec(&mut dtb_mem, 0x8000_0000, &dtb);
    let persistent = dtb_mem.into_data();

    let mem = ByteMap::default().with_persistent(persistent.clone());
    let mut cpu = Processor::new(mem);

    let mut last_insn: Option<Insn> = None;
    let mut last_state: Option<State> = None;
    let mut last_store: Option<MemoryTrace> = None;
    let mut last_mems = vec![];

    let mut counter = 0;

    use std::time::SystemTime;
    let mut mark = SystemTime::now();

    for (step, log_tuple) in logs.enumerate() {
        let LogTuple {
            line,
            state,
            insn,
            store,
            mems,
        } = log_tuple;
        counter += 1;

        if Some(&state) == last_state.as_ref() {
            debug!("Last state and this state are the same");
            continue;
        }
        if insn.as_ref() == last_insn.as_ref() {
            warn!("Last insn and this insn are the same");
        }

        test_state(
            matchers,
            &last_state,
            &last_insn,
            &last_mems,
            &state,
            &last_store,
        );
        trace!("Transaction validated OK");

        // trace!("This insn {:?}", insn);

        if step % 100_0000 == 0 {
            warn!("Step {}mil ({})", (step as f32) / 1_000_000.0, step);
        }

        last_insn = insn;
        last_state = Some(state);
        last_store = store;
        last_mems = mems;
        continue;

        let mut fail = false;

        // validate current state

        if let Some(ref insn) = &insn {
            if insn.pc != state.pc {
                error!("Insn and state pc do not match");
                fail = true;
            }
        }

        if !state.validate(&cpu, None as Option<State>) {
            fail = true;
        }

        // check for memory store
        if let Some(store) = last_store {
            if !store.validate(cpu.mmu_mut()) {
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

        if step % 10_0000 == 0 {
            let d = SystemTime::now().duration_since(mark).expect("time");
            let in_ms = d.as_secs() * 1000 + d.subsec_nanos() as u64 / 1_000_000;
            let insn_per_sec = 10_000f32 / (in_ms as f32 / 1000f32);
            let insn_per_sec = insn_per_sec as u32;

            warn!("{} - Speed: {:?} hz", status_line, insn_per_sec);
            mark = SystemTime::now();
        }
        info!("{}", status_line);

        for mem in &mems {
            cpu.mmu_mut().mem_mut().write_b(mem.addr, mem.value as u8);
        }

        cpu.step(matchers);

        last_insn = insn;
        last_state = Some(state);
        last_store = store;
        last_mems = mems;
    }

    warn!("Retired {} insns", counter);
    Ok(())
}
