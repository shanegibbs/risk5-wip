use self::logtupleiterator::*;
use crate::regs;
use crate::Processor;
use serde_json;
use std::io;
use std::io::BufReader;
use std::io::Lines;

mod loglineiterator;
// mod logtuplededupiterator;
mod logtupleiterator;
mod run;

pub use self::run::run;

pub fn convert() -> Result<(), io::Error> {
    use bincode;
    use std::io::BufWriter;

    let mut out = BufWriter::new(io::stdout());

    error!("starting");

    for line in JsonLogTupleIterator::new()? {
        trace!("{:?}", line);
        let bin = line.to_logtuple();
        bincode::serialize_into(&mut out, &bin).expect("serialize");
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum LogLine {
    #[serde(rename = "mark")]
    Mark,
    #[serde(rename = "insn")]
    Insn(JsonInsn),
    #[serde(rename = "state")]
    State(JsonState),
    #[serde(rename = "load")]
    Load(JsonMemory),
    #[serde(rename = "store")]
    Store(JsonMemory),
    #[serde(rename = "mem")]
    Memory(JsonMemory),
}

fn string_to_u64(s: &String) -> u64 {
    u64::from_str_radix(&s[2..], 16).expect("hex parse")
}

fn string_to_u32(s: &String) -> u32 {
    u32::from_str_radix(&s[2..], 16).expect("hex parse")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Insn {
    pc: u64,
    bits: u32,
    desc: String,
}

impl Into<Insn> for JsonInsn {
    fn into(self) -> Insn {
        Insn {
            pc: string_to_u64(&self.pc),
            bits: string_to_u32(&self.bits),
            desc: self.desc,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonInsn {
    core: usize,
    pc: String,
    bits: String,
    desc: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    id: usize,
    pc: u64,
    prv: u64,

    mstatus: u64,
    mepc: u64,
    mtvec: u64,
    mscratch: u64,

    mcause: u64,
    xregs: Vec<u64>,
}

impl Into<State> for JsonState {
    fn into(self) -> State {
        State {
            id: self.id,
            pc: string_to_u64(&self.pc),
            prv: string_to_u64(&self.prv),
            mstatus: string_to_u64(&self.mstatus),
            mcause: string_to_u64(&self.mcause),
            mscratch: string_to_u64(&self.mscratch),
            mtvec: string_to_u64(&self.mtvec),
            mepc: string_to_u64(&self.mepc),
            xregs: self.xregs.iter().map(|n| string_to_u64(n)).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonState {
    id: usize,
    pc: String,
    prv: String,

    mstatus: String,
    mepc: String,
    mtval: String,
    mscratch: String,
    mtvec: String,

    // mideleg: String,
    mcause: String,
    xregs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Memory {
    kind: String,
    addr: u64,
    value: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonMemory {
    #[serde(rename = "type")]
    kind: String,
    addr: String,
    value: String,
}

impl Into<Memory> for JsonMemory {
    fn into(self) -> Memory {
        let JsonMemory { kind, addr, value } = self;
        Memory {
            kind: kind,
            addr: string_to_u64(&addr),
            value: string_to_u64(&value),
        }
    }
}

// impl Into<FakeMemoryItem> for Memory {
//     fn into(self) -> FakeMemoryItem {
//         let addr = u64::from_str_radix(&self.addr[2..], 16).expect("load value)");
//         let value = &self.value[2..];
//         let value = u64::from_str_radix(value, 16).expect("load value)");
//         if self.kind == "int64" || self.kind == "uint64" {
//             FakeMemoryItem::Double(addr, value)
//         } else if self.kind == "int32" || self.kind == "uint32" {
//             FakeMemoryItem::Word(addr, value as u32)
//         } else if self.kind == "uint8" {
//             FakeMemoryItem::Byte(addr, value as u8)
//         } else {
//             unimplemented!("load fake val");
//         }
//     }
// }

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JsonLogTuple {
    line: usize,
    state: JsonState,
    insn: JsonInsn,
    mems: Vec<JsonMemory>,
}

impl JsonLogTuple {
    fn to_logtuple(self) -> LogTuple {
        LogTuple {
            line: self.line,
            state: self.state.into(),
            insn: self.insn.into(),
            mems: self.mems.into_iter().map(|n| n.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LogTuple {
    line: usize,
    state: State,
    insn: Insn,
    mems: Vec<Memory>,
}
