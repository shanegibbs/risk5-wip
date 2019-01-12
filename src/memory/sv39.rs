use super::Memory;
use std::fmt;

pub(crate) struct Sv39<M> {
    mem: M,
    asid: u64,
    ppn: u64,
}

impl<M: Memory> Memory for Sv39<M> {
    fn read_b(&self, offset: u64) -> u8 {
        self.mem.read_b(offset)
    }

    fn write_b(&mut self, offset: u64, value: u8) {
        self.mem.write_b(offset, value)
    }
}

impl<M> fmt::Debug for Sv39<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sv39")
    }
}
