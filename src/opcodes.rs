use crate::mmu::*;
use crate::Processor;
use std::fmt;

pub struct Matcher<M: Memory> {
    mask: u32,
    mtch: u32,
    exec: fn(&mut Processor<M>, u32),
}

impl<M: Memory> Matcher<M> {
    pub fn new(mask: u32, mtch: u32, exec: fn(&mut Processor<M>, u32)) -> Self {
        Self { mask, mtch, exec }
    }
    pub fn matches(&self, insn: u32) -> bool {
        insn & self.mask == self.mtch
    }
    pub fn exec(&self, p: &mut Processor<M>, insn: u32) {
        (self.exec)(p, insn)
    }
}

impl<M: fmt::Debug + Memory> fmt::Debug for Matcher<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Matcher")
    }
}

pub static REG_NAMES: &'static [&str] = &[
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "sA", "sB", "t3", "t4", "t5",
    "t6",
];

#[derive(Debug)]
pub(crate) struct Regs {
    regs: [u64; 32],
}

impl Regs {
    pub fn new() -> Self {
        Regs { regs: [0; 32] }
    }

    #[inline(always)]
    pub fn get<T: Into<usize>>(&self, i: T) -> u64 {
        let i = i.into();
        let v = self.regs[i];
        // trace!("Getting reg 0x{:x} 0x{:x}", i, v);
        v
    }

    #[inline(always)]
    pub fn geti<T: Into<usize>>(&self, i: T) -> i64 {
        let i = i.into();
        let v = self.regs[i];
        // trace!("Getting reg 0x{:x} 0x{:x}", i, v);
        v as i64
    }

    #[inline(always)]
    pub fn set<T: Into<usize>>(&mut self, i: T, v: u64) {
        let i = i.into();
        // reg 0 is a black hole
        if i == 0 {
            return;
        }
        trace!("Setting reg 0x{:x} 0x{:x}", i, v);
        self.regs[i] = v;
    }

    #[inline(always)]
    pub fn seti<T: Into<usize>>(&mut self, i: T, v: i64) {
        self.set(i, v as u64)
    }
}
