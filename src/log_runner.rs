use crate::memory::*;
use crate::regs;
use crate::Processor;
use pretty_env_logger;
use serde_json;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Lines;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
enum LogLine {
    #[serde(rename = "mark")]
    Mark,
    #[serde(rename = "insn")]
    Insn(Insn),
    #[serde(rename = "state")]
    State(State),
    #[serde(rename = "load")]
    Load(Memory),
    #[serde(rename = "store")]
    Store(Memory),
    #[serde(rename = "mem")]
    Memory(Memory),
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
    // mideleg: String,
    mcause: String,
    mscratch: String,
    mtvec: String,
    mepc: String,
    xregs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Memory {
    #[serde(rename = "type")]
    kind: String,
    addr: String,
    value: String,
}

impl Into<FakeMemoryItem> for Memory {
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
    count: usize,
    had_first_state: bool,
    lines: Lines<BufReader<io::Stdin>>,
}

impl LogLineIterator {
    fn new() -> Result<Self, io::Error> {
        let reader = BufReader::new(io::stdin());
        Ok(LogLineIterator {
            count: 0,
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

            self.count += 1;

            let l = line.expect("line");

            if l.is_empty() {
                continue;
            }

            let d: LogLine = match serde_json::from_str(l.as_str()) {
                Ok(l) => l,
                Err(e) => {
                    error!("Parsing line: {}", e);
                    error!("{}", l);
                    panic!("line parse failed");
                }
            };

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

struct LogTuple {
    line: usize,
    state: State,
    insn: Insn,
    load: Option<Memory>,
    store: Option<Memory>,
    mems: Vec<Memory>,
}

struct LogTupleIterator {
    line_it: LogLineIterator,
}

impl LogTupleIterator {
    fn new() -> Result<Self, io::Error> {
        let mut it = LogLineIterator::new()?;
        loop {
            match it.next() {
                None => break,
                Some(LogLine::Mark) => break,
                _ => (),
            }
        }

        Ok(LogTupleIterator { line_it: it })
    }
}

impl Iterator for LogTupleIterator {
    type Item = LogTuple;

    fn next(&mut self) -> Option<LogTuple> {
        let mut insn = None;
        let mut state = None;
        let mut load = None;
        let mut store = None;
        let mut mems = vec![];

        loop {
            match self.line_it.next() {
                Some(LogLine::Mark) => {
                    if insn.is_some() {
                        break;
                    }
                }
                Some(LogLine::Insn(n)) => insn = Some(n),
                Some(LogLine::Load(n)) => load = Some(n),
                Some(LogLine::Store(n)) => store = Some(n),
                Some(LogLine::State(n)) => state = Some(n),
                Some(LogLine::Memory(n)) => mems.push(n),
                None => return None,
            }
        }

        let insn = insn.expect(&format!("insn. line {}", self.line_it.count));
        let state = state.expect("state");

        Some(LogTuple {
            line: self.line_it.count,
            state: state,
            insn,
            load,
            store,
            mems,
        })
    }
}

struct LogTupleDedupIterator {
    it: LogTupleIterator,
    buf: Option<LogTuple>,
}

impl LogTupleDedupIterator {
    fn new() -> Result<Self, io::Error> {
        Ok(LogTupleDedupIterator {
            it: LogTupleIterator::new()?,
            buf: None,
        })
    }
}

impl Iterator for LogTupleDedupIterator {
    type Item = LogTuple;

    fn next(&mut self) -> Option<LogTuple> {
        if self.buf.is_none() {
            let n = self.it.next();
            if n.is_none() {
                return None;
            }
            self.buf = n;
            return self.next();
        }

        let buf = self.buf.take().expect("buf");
        let n = match self.it.next() {
            Some(n) => n,
            None => return Some(buf),
        };

        if n.state.pc == buf.state.pc {
            panic!("duplicate insn pc");
            // merge
            let mut n = n;
            n.mems.extend(buf.mems);
            return Some(n);
        } else {
            self.buf = Some(n);
            return Some(buf);
        }
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

    let matchers = crate::build_matchers::<ByteMap>();

    let dtb = crate::load_dtb();

    let mem = ByteMap::new();
    let mut cpu = Processor::new(0x1000, mem);
    let mut last_insn: Option<Insn> = None;

    info!("Initial checks");

    for (step, log_tuple) in LogTupleDedupIterator::new()?.enumerate() {
        let LogTuple {
            line,
            state,
            insn,
            load,
            store,
            mems,
        } = log_tuple;
        // trace!("{:?}", state);

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

        if cpu.pc() != u64::from_str_radix(&state.pc[2..], 16).expect("pc") {
            error!(
                "Fail pc check. Was: 0x{:016x}, expected: {}",
                cpu.pc(),
                state.pc
            );
            fail = true;
        }

        fail_on!("prv", state.prv, cpu.csrs().prv);
        fail_on!("mepc", state.mepc, cpu.csrs().mepc);
        fail_on!("mcause", state.mcause, cpu.csrs().mcause);
        fail_on!("mscratch", state.mscratch, cpu.csrs().mscratch);
        fail_on!("mtvec", state.mtvec, cpu.csrs().mtvec);
        // fail_on!("mideleg", state.mideleg, cpu.csrs().mideleg);

        {
            let val = u64::from_str_radix(&state.mstatus[2..], 16).expect("mstatus");
            if val != cpu.csrs().mstatus.val() {
                use crate::bitfield::Mstatus;
                let mstatus_expected = Mstatus::new_with_val(val);
                error!(
                    "Fail mstatus check\n{}\nWas:      {:?}\nExpected: {:?}",
                    format_diff(val, cpu.csrs().mstatus.val()),
                    cpu.csrs().mstatus,
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
                    i, regs::REG_NAMES[i], actual, actual, val, val);
                error!("{}", msg);
                fail = true;
            }
        }

        // if cpu.fake_mem().queue_size() != 0 {
        //     // if mem.addr != "0x80009000" && mem.addr != "0x80009008" {}
        //     warn!("Memory operations still queued");
        //     // fail = true;
        // }
        cpu.mmu_mut().mem_mut().clear();
        crate::write_reset_vec(cpu.mmu_mut().mem_mut(), 0x80000000, &dtb);

        if fail {
            let last_insn = last_insn.expect("last_insn");
            error!("debug info - step: {}", step - 1);
            error!("debug info - PC:   {}", last_insn.pc);
            error!("debug info - Insn: {}", last_insn.desc);
            error!("debug info - Line: {}", line);
            panic!("Failed checks");
        }

        // load up transactions

        if step % 10000 == 0 {
            warn!("--- Begin step {} ({}) ---", step, line);
        }
        info!("--- Begin step {} ({}) ---", step, line);

        debug!("{:?}", insn);

        // let insn_pc = u64::from_str_radix(&insn.pc[2..], 16).expect("pc");
        // let insn_bits = u32::from_str_radix(&insn.bits[2..], 16).expect("insn bits");

        // if let Some(mem) = load {
        //     trace!("Load {:?}", mem);
        //     cpu.fake_mem().push_read(mem);
        // }

        // if let Some(mem) = store {
        //     trace!("Store {:?}", mem);
        //     cpu.fake_mem().push_write(mem);
        // }

        // cpu.fake_mem()
        //     .push_read(FakeMemoryItem::Word(insn_pc, insn_bits));

        trace!("Have {} mems", mems.len());
        for mem in mems {
            trace!("{:?}", mem);
            use crate::Memory;
            cpu.mmu_mut().mem_mut().write_b(
                u64::from_str_radix(&mem.addr[2..], 16).expect("mem.addr"),
                u8::from_str_radix(&mem.value[2..], 16).expect("mem.value"),
            );
        }

        for (addr, value) in &cpu.mmu().mem().data {
            if *addr >= 0x4096 {
                trace!("Have 0x{:x}: 0x{:x}", addr, value);
            }
        }

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
