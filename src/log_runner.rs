use crate::mmu::*;
use crate::opcodes;
use pretty_env_logger;
use serde_json;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Lines;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
enum LogLine {
    #[serde(rename = "insn")]
    Insn(Insn),
    #[serde(rename = "state")]
    State(State),
    #[serde(rename = "load")]
    Load(Load),
}

#[derive(Serialize, Deserialize, Debug)]
struct Insn {
    core: usize,
    pc: String,
    bits: String,
    desc: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct State {
    id: usize,
    pc: String,
    prv: String,
    mstatus: String,
    mcause: String,
    mscratch: String,
    mtvec: String,
    mepc: String,
    xregs: Vec<String>,
    fregs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Load {
    #[serde(rename = "type")]
    kind: String,
    addr: String,
    value: String,
}

impl Into<FakeMemoryItem> for Load {
    fn into(self) -> FakeMemoryItem {
        let addr = u64::from_str_radix(&self.addr[2..], 16).expect("load value)");
        let value = &self.value[2..];
        let value = u64::from_str_radix(value, 16).expect("load value)");
        if self.kind == "int64" || self.kind == "uint64" {
            FakeMemoryItem::Double(addr, value)
        } else if self.kind == "int32" || self.kind == "uint32" {
            FakeMemoryItem::Word(addr, value as u32)
        } else if self.kind == "uint8" {
            FakeMemoryItem::Byte(addr, value as u8)
        } else {
            unimplemented!("load fake val");
        }
    }
}

struct LogLineIterator {
    had_first_state: bool,
    lines: Lines<BufReader<io::Stdin>>,
}

impl LogLineIterator {
    fn new() -> Result<Self, io::Error> {
        let reader = BufReader::new(io::stdin());
        Ok(LogLineIterator {
            had_first_state: false,
            lines: reader.lines(),
        })
    }
}

impl Iterator for LogLineIterator {
    type Item = LogLine;

    fn next(&mut self) -> Option<LogLine> {
        loop {
            let line = match self.lines.next() {
                Some(l) => l,
                None => return None,
            };

            let l = line.unwrap();

            if l.is_empty() {
                continue;
            }

            let d: LogLine = serde_json::from_str(l.as_str()).unwrap();

            if let LogLine::State(_) = d {
                self.had_first_state = true;
            } else {
                if !self.had_first_state {
                    continue;
                }
            }

            return Some(d);
        }
    }
}

struct LogTupleIterator {
    line_it: LogLineIterator,
    next_state: Option<State>,
}

impl LogTupleIterator {
    fn new() -> Result<Self, io::Error> {
        let mut lines = LogLineIterator::new()?;

        let next_state = loop {
            let line = lines.next();
            if let Some(LogLine::State(s)) = line {
                break s;
            }
        };

        Ok(LogTupleIterator {
            line_it: lines,
            next_state: Some(next_state),
        })
    }
}

impl Iterator for LogTupleIterator {
    type Item = (State, Insn, Option<Load>);

    fn next(&mut self) -> Option<(State, Insn, Option<Load>)> {
        self.next_state.take().and_then(|state| {
            let mut insn = None;
            let mut load = None;

            loop {
                match self.line_it.next() {
                    Some(LogLine::Insn(n)) => insn = Some(n),
                    Some(LogLine::Load(n)) => load = Some(n),
                    Some(LogLine::State(n)) => {
                        let pc = n.pc.clone();
                        self.next_state = Some(n);

                        // sometimes spike restarts instrucions
                        if state.pc == pc {
                            return self.next();
                        } else {
                            break;
                        }
                    }
                    None => return None,
                }
            }
            let insn = insn.unwrap();

            Some((state, insn, load))
        })
    }
}

fn format_diff(expected: u64, actual: u64) -> String {
    format!(
        "Was:      0x{:016x} {:064b}\nExpected: 0x{:016x} {:064b}",
        actual, actual, expected, expected
    )
}

fn run_err() -> Result<(), io::Error> {
    pretty_env_logger::init();

    let matchers = crate::build_matchers::<FakeMemory>();

    let mem = FakeMemory::new();
    let mut cpu = opcodes::Processor::new(0x1000, mem);
    let mut last_insn: Option<Insn> = None;

    info!("Initial checks");

    for (step, (state, insn, load)) in LogTupleIterator::new()?.enumerate() {
        // trace!("{:?}", state);

        let mut fail = false;

        /* for i in 0..8 {
            let offset = (i * 4) as usize;
            debug!("0x{:16x} 0x{:16x} 0x{:16x} 0x{:16x}", cpu.regs.get(offset), cpu.regs.get(offset + 1), cpu.regs.get(offset + 2), cpu.regs.get(offset + 3));
        } */

        // validate current state

        macro_rules! fail_on {
            ($name:expr, $expected:expr, $actual:expr) => {
                let val = u64::from_str_radix(&$expected[2..], 16).expect($name);
                if val != $actual {
                    error!("Fail check on {}.\n{}", $name, format_diff(val, $actual));
                    fail = true;
                }
            };
        }

        macro_rules! warn_on {
            ($name:expr, $expected:expr, $actual:expr) => {
                let val = u64::from_str_radix(&$expected[2..], 16).expect($name);
                if val != $actual {
                    warn!(
                        "Fail {} check. Was: 0x{:016x}, expected: {}",
                        $name, $actual, $expected
                    );
                }
            };
        }

        if cpu.pc() != u64::from_str_radix(&state.pc[2..], 16).expect("pc") {
            error!(
                "Fail pc check. Was: 0x{:016x}, expected: {}",
                cpu.pc(),
                state.pc
            );
            fail = true;
        }

        fail_on!("prv", state.prv, cpu.csrs.prv);
        warn_on!("mepc", state.mepc, cpu.csrs.mepc);
        fail_on!("mcause", state.mcause, cpu.csrs.mcause);
        fail_on!("mscratch", state.mscratch, cpu.csrs.mscratch);
        fail_on!("mtvec", state.mtvec, cpu.csrs.mtvec);

        {
            let val = u64::from_str_radix(&state.mstatus[2..], 16).expect("mstatus");
            if val != cpu.csrs.mstatus.val() {
                use crate::mstatus::Mstatus;
                let mstatus_expected = Mstatus::new_with_val(val);
                error!(
                    "Fail mstatus check\n{}\nWas:      {:?}\nExpected: {:?}",
                    format_diff(val, cpu.csrs.mstatus.val()),
                    cpu.csrs.mstatus,
                    mstatus_expected
                );
                fail = true;
            }
        }

        for (i, reg_str) in state.xregs.iter().enumerate() {
            let val = u64::from_str_radix(&reg_str[2..], 16).expect("xreg");
            let actual = cpu.regs.get(i);
            if val != actual {
                let msg = format!("Fail reg check on 0x{:02x} ({})\nWas:      0x{:016x} {:064b} \nExpected: 0x{:016x} {:064b}",
                    i, opcodes::REG_NAMES[i], actual, actual, val, val);
                error!("{}", msg);
                fail = true;
            }
        }

        if fail {
            let last_insn = last_insn.expect("last_insn");
            error!("step: {}", step);
            error!("PC:   {}", last_insn.pc);
            error!("Insn: {}", last_insn.desc);
            panic!("Failed checks");
        }

        // stop if required
        {
            use std::env;
            if let Ok(val) = env::var("STOP_AT") {
                let current_step = format!("{}", step);
                if val == current_step {
                    return Ok(());
                }
            }
        }

        // load up transactions

        info!("--- Begin step {} ---", step);

        debug!("{:?}", insn);
        trace!("Load {:?}", load);

        let insn_pc = u64::from_str_radix(&insn.pc[2..], 16).expect("pc");
        let insn_bits = u32::from_str_radix(&insn.bits[2..], 16).expect("insn bits");

        if let Some(load) = load {
            cpu.get_mem().push(load);
        }

        cpu.get_mem().push(FakeMemoryItem::Word(insn_pc, insn_bits));
        cpu.step(&matchers);

        last_insn = Some(insn);
    }

    Ok(())
}

pub fn run() {
    if let Err(e) = run_err() {
        error!("{}", e)
    }
}
