use self::logtupleiterator::*;
use crate::regs;
use crate::Processor;
use serde_json;
use std::io::BufReader;
use std::io::Lines;
use std::{fmt, io};

pub(crate) mod json;
mod logger;
mod loglineiterator;
mod logtupleiterator;
mod run;

pub use self::run::convert;
pub use self::run::run;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LogTuple {
    line: usize,
    state: State,
    insn: Option<Insn>,
    store: Option<MemoryTrace>,
    mems: Vec<MemoryTrace>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Insn {
    pc: u64,
    bits: u32,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryTrace {
    kind: String,
    addr: u64,
    value: u64,
}

impl fmt::Display for Insn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pc=0x{:x} bits=0x{:x} {}", self.pc, self.bits, self.desc)
    }
}

impl fmt::Display for MemoryTrace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "kind={} addr=0x{:x} value=0x{:x}",
            self.kind, self.addr, self.value
        )
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
