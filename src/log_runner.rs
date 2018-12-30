use mmu::*;
use opcodes;
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
        let mut lines = try!(LogLineIterator::new());

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
                            return self.next()
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
    format!("Was:      0x{:016x} {:064b}\nExpected: 0x{:016x} {:064b}", actual, actual, expected, expected)
}

fn run_err() -> Result<(), io::Error> {
    pretty_env_logger::init();

    let matchers = ::build_matchers::<FakeMemory>();

    let mem = FakeMemory::new();
    let mut cpu = opcodes::Processor::new(0x1000, mem);

    info!("Initial checks");

    for (step, (state, insn, load)) in try!(LogTupleIterator::new()).enumerate() {
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
            }
        }

        macro_rules! warn_on {
            ($name:expr, $expected:expr, $actual:expr) => {
                let val = u64::from_str_radix(&$expected[2..], 16).expect($name);
                if val != $actual {
                    warn!("Fail {} check. Was: 0x{:016x}, expected: {}", $name, $actual, $expected);
                }
            }
        }

        if cpu.pc() != u64::from_str_radix(&state.pc[2..], 16).expect("pc") {
            error!("Fail pc check. Was: 0x{:016x}, expected: {}", cpu.pc(), state.pc);
            fail = true;
        }

        fail_on!("prv", state.prv, cpu.csrs.prv);
        warn_on!("mepc", state.mepc, cpu.csrs.mepc);
        fail_on!("mcause", state.mcause, cpu.csrs.mcause);

        {
            let val = u64::from_str_radix(&state.mstatus[2..], 16).expect("mstatus");
            if val != cpu.csrs.mstatus.val() {
                use mstatus::Mstatus;
                let mstatus_expected = Mstatus::new_with_val(val);
                error!("Fail mstatus check\n{}\nWas:      {:?}\nExpected: {:?}",
                       format_diff(val, cpu.csrs.mstatus.val()), cpu.csrs.mstatus, mstatus_expected);
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
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed checks")));
        }

        // ratchet
        if step > 144 {
            // break;
        }

        // load up transactions

        info!("--- Begin step {} ---", step);

        debug!("{:?}", insn);
        trace!("Load {:?}", load);

        let insn_bits = u32::from_str_radix(&insn.bits[2..], 16).expect("insn bits");

        if let Some(load) = load {
            let load_val = u64::from_str_radix(&load.value[2..], 16).expect("load value)");
            cpu.get_mem().push_double(load_val);
        }

        cpu.get_mem().push_word(insn_bits);
        cpu.step(&matchers);
    }

    Ok(())
}

pub fn run() {
    if let Err(e) = run_err() {
        error!("{}", e)
    }
}
