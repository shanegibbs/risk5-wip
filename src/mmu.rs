use crate::Memory;
use std::fmt;

pub struct Mmu<M> {
    mem: M,
}

impl<M> Mmu<M> {
    pub fn new(m: M) -> Self {
        Self { mem: m }
    }
    pub fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }
    pub fn bare(&self) -> &M {
        &self.mem
    }
    pub fn bare_mut(&mut self) -> &mut M {
        &mut self.mem
    }
}

impl<M: Memory> Memory for Mmu<M> {
    fn read_b(&self, offset: u64) -> u8 {
        self.mem.read_b(offset)
    }

    fn read_h(&self, offset: u64) -> u16 {
        self.mem.read_h(offset)
    }

    fn read_w(&self, offset: u64) -> u32 {
        self.mem.read_w(offset)
    }

    fn read_d(&self, offset: u64) -> u64 {
        self.mem.read_d(offset)
    }

    fn write_b(&mut self, offset: u64, value: u8) {
        self.mem.write_b(offset, value)
    }

    fn write_h(&mut self, offset: u64, value: u16) {
        self.mem.write_h(offset, value)
    }

    fn write_w(&mut self, offset: u64, value: u32) {
        self.mem.write_w(offset, value)
    }

    fn write_d(&mut self, offset: u64, value: u64) {
        self.mem.write_d(offset, value)
    }
}

impl<M> fmt::Debug for Mmu<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mmu")
    }
}
