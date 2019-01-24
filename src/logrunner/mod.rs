use crate::mmu::Mmu;
use crate::regs;
use crate::Memory;
use serde_json;
use std::default::Default;
use std::io::BufReader;
use std::io::Lines;
use std::{fmt, io};

mod bincode;
pub(crate) mod json;
mod logger;
mod logtupleiterator;
mod run;

pub use self::bincode::convert;
pub use self::run::run;

pub(crate) fn format_diff<T: fmt::Binary + fmt::LowerHex>(expected: T, actual: T) -> String {
    let hex_width = format!("{:x}", actual)
        .len()
        .max(format!("{:x}", actual).len());
    let binary_width = format!("{:b}", actual)
        .len()
        .max(format!("{:b}", actual).len());

    format!(
        "Was:      0x{2:00$x} {2:01$b}\nExpected: 0x{3:00$x} {3:01$b}",
        hex_width, binary_width, actual, expected
    )
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LogTuple {
    line: usize,
    state: State,
    insn: Option<Insn>,
    store: Option<MemoryTrace>,
    mems: Vec<MemoryTrace>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Insn {
    pc: u64,
    bits: u32,
    desc: String,
}

pub struct RestorableState<'s, M> {
    pub state: &'s State,
    pub memory: M,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub(crate) id: usize,
    pub(crate) pc: u64,
    pub(crate) prv: u64,

    pub(crate) mstatus: u64,
    pub(crate) mepc: u64,
    pub(crate) mtvec: u64,
    pub(crate) mcause: u64,
    pub(crate) mscratch: u64,
    pub(crate) minstret: u64,
    pub(crate) mie: u64,
    pub(crate) mip: u64,
    pub(crate) medeleg: u64,
    pub(crate) mideleg: u64,
    pub(crate) mcounteren: u64,
    pub(crate) scounteren: u64,
    pub(crate) sepc: u64,
    pub(crate) stval: u64,
    pub(crate) sscratch: u64,
    pub(crate) stvec: u64,
    pub(crate) satp: u64,
    pub(crate) scause: u64,

    pub(crate) xregs: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MemoryTrace {
    kind: String,
    addr: u64,
    value: u64,
}

trait ToMemory {
    fn to_memory<M: Memory + Default>(&self) -> M;
}

impl ToMemory for Vec<MemoryTrace> {
    fn to_memory<M: Memory + Default>(&self) -> M {
        let mut memory: M = Default::default();
        for mem in self {
            memory.write_b(mem.addr, mem.value as u8);
        }
        memory
    }
}

impl MemoryTrace {
    fn validate<M>(&self, m: &Mmu<M>) -> bool
    where
        M: Memory,
    {
        trace!("Checking store {}", self);
        if self.addr >= 0x8000_9000 && self.addr < 0x8000_9016 {
            trace!("Ignoring write to HTIF: {}", self);
            return true;
        }

        macro_rules! fail_on {
            ($name:expr, $expected:expr, $actual:expr) => {
                if $expected != $actual {
                    error!(
                        "Fail check on {}.\n{}",
                        $name,
                        format_diff($expected, $actual)
                    );
                    return false;
                }
                return true;
            };
        }

        match self.kind.as_str() {
            "uint8" => {
                let val = m.read_b(self.addr).expect("mmu read");
                fail_on!("store", self.value as u8, val);
            }
            "uint16" => {
                let val = m.read_h(self.addr).expect("mmu read");
                fail_on!("store", self.value as u16, val);
            }
            "uint32" => {
                let val = m.read_w(self.addr).expect("mmu read");
                fail_on!("store", self.value as u32, val);
            }
            "uint64" => {
                let val = m.read_d(self.addr).expect("mmu read");
                fail_on!("store", self.value, val);
            }
            n => unimplemented!("check store type {}", n),
        }
    }
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

impl State {
    fn validate<S>(&self, other: S) -> bool
    where
        S: Into<State>,
    {
        let other = other.into();

        macro_rules! valid {
            ($id:ident) => {{
                let actual = other.$id;
                let expected = self.$id;
                if actual != expected {
                    error!(
                        "Fail check on {}.\n{}",
                        stringify!($id),
                        format_diff(expected, actual)
                    );
                    false
                } else {
                    true
                }
            }};
            ($id:ident, $($next:ident),+) => {{
                valid!($id) & valid!($($next),+)
            }};
        }

        // TODO add mip mcounteren
        // TODO add supervisor csrs
        let mut valid =
            valid!(pc, prv) & valid!(mepc, mtvec, mcause, mscratch, mie, medeleg, mideleg);

        if self.mstatus != other.mstatus {
            use crate::bitfield::Mstatus;
            let mstatus_expected = Mstatus::new_with_val(self.mstatus);
            let mstatus_actual = Mstatus::new_with_val(other.mstatus);
            error!(
                "Fail mstatus check\n{}\nWas:      {:?}\nExpected: {:?}",
                format_diff(self.mstatus, other.mstatus),
                mstatus_actual,
                mstatus_expected
            );
            valid = false;
        }

        for (i, reg) in self.xregs.iter().enumerate() {
            let actual = other.xregs[i];
            if *reg != actual {
                let msg = format!("Fail reg check on 0x{:02x} ({})\nWas:      0x{:016x} {:064b} \nExpected: 0x{:016x} {:064b}",
                    i, regs::REG_NAMES[i], actual, actual, reg, reg);
                error!("{}", msg);
                valid = false;
            }
        }

        valid
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
