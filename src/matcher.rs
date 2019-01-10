use crate::Memory;
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
