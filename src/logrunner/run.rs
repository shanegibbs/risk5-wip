use super::*;
use crate::memory::*;
use log::Level::*;
use std::panic;

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

fn format_diff(expected: u64, actual: u64) -> String {
    format!(
        "Was:      0x{:016x} {:064b}\nExpected: 0x{:016x} {:064b}",
        actual, actual, expected, expected
    )
}

pub fn run() -> Result<(), io::Error> {
    super::logger::init().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    internal_run()
}

fn internal_run() -> Result<(), io::Error> {
    warn!("Loading log");

    let reader = BincodeReader::new();

    use std::env;
    if let Ok(val) = env::var("STOP_AT") {
        let stop_at = val.parse::<usize>().expect("parse STOP_AT");
        return run_log(reader.take(stop_at).collect());
    }

    run_log(reader.collect())
}

#[inline(never)]
fn run_log(logs: Vec<LogTuple>) -> Result<(), io::Error> {
    warn!("Beginning log run");

    let matchers = crate::build_matchers::<ByteMap>();

    let mem = ByteMap::new();
    let mut cpu = Processor::new(0x1000, mem);
    let mut last_insn: Option<Insn> = None;

    let dtb = crate::load_dtb();
    crate::write_reset_vec(cpu.mmu_mut().mem_mut(), 0x80000000, &dtb);

    for (step, log_tuple) in logs.into_iter().enumerate() {
        let LogTuple {
            line,
            state,
            insn,
            mems,
        } = log_tuple;

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

        /* macro_rules! warn_on {
            ($name:expr, $expected:expr, $actual:expr) => {
                let val = u64::from_str_radix(&$expected[2..], 16).expect($name);
                if val != $actual {
                    warn!(
                        "Fail {} check. Was: 0x{:016x}, expected: {}",
                        $name, $actual, $expected
                    );
                }
            };
        } */

        if let Some(ref insn) = &insn {
            if insn.pc != state.pc {
                error!("Insn and state pc do not match");
                fail = true;
            }
        }

        fail_on!("pc", state.pc, cpu.pc());
        fail_on!("prv", state.prv, cpu.csrs().prv);
        fail_on!("mepc", state.mepc, cpu.csrs().mepc);
        fail_on!("mcause", state.mcause, cpu.csrs().mcause);
        fail_on!("mscratch", state.mscratch, cpu.csrs().mscratch);
        fail_on!("mtvec", state.mtvec, cpu.csrs().mtvec);
        // fail_on!("mideleg", state.mideleg, cpu.csrs().mideleg);

        {
            if state.mstatus != cpu.csrs().mstatus.val() {
                use crate::bitfield::Mstatus;
                let mstatus_expected = Mstatus::new_with_val(state.mstatus);
                error!(
                    "Fail mstatus check\n{}\nWas:      {:?}\nExpected: {:?}",
                    format_diff(state.mstatus, cpu.csrs().mstatus.val()),
                    cpu.csrs().mstatus,
                    mstatus_expected
                );
                fail = true;
            }
        }

        for (i, reg) in state.xregs.iter().enumerate() {
            let actual = cpu.regs.get(i);
            if *reg != actual {
                let msg = format!("Fail reg check on 0x{:02x} ({})\nWas:      0x{:016x} {:064b} \nExpected: 0x{:016x} {:064b}",
                    i, regs::REG_NAMES[i], actual, actual, reg, reg);
                error!("{}", msg);
                fail = true;
            }
        }

        if fail {
            let last_insn = last_insn.expect("last_insn");
            error!("debug info - step:     {}", step - 1);
            error!(
                "debug info - Insn PC:  0x{:x}",
                insn.map(|i| i.pc).unwrap_or(0)
            );
            error!("debug info - State PC: 0x{:x}", state.pc);
            error!("debug info - Insn:     {}", last_insn.desc);
            error!("debug info - Line:     {}", line);
            panic!("Failed checks");
        }

        trace!("State validated OK from: {:?}", last_insn);

        // reset and begin again

        cpu.mmu_mut().mem_mut().clear();

        if step % 10000 == 0 {
            warn!("--- Begin step {} ({}) ---", step, line);
        }

        if let Some(ref insn) = insn {
            info!("--- Begin step {} ({}) --- {}", step, line, insn);
        } else {
            info!("--- Begin step {} ({}) --- No insn", step, line);
        }

        for mem in mems {
            use crate::Memory;
            cpu.mmu_mut().mem_mut().write_b(mem.addr, mem.value as u8);
        }

        if log_enabled!(Trace) {
            for (addr, value) in &cpu.mmu().mem().data {
                if *addr >= 0x4096 {
                    trace!("Have 0x{:x}: 0x{:x}", addr, value);
                }
            }
        }

        cpu.step(&matchers);

        last_insn = insn;
    }

    Ok(())
}
