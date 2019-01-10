use crate::csrs::Csrs;
use crate::mmu::FakeMemory;
use crate::opcodes::*;
use crate::Memory;

#[derive(Debug)]
pub struct Processor<M> {
    pc: u64,
    pub(crate) regs: Regs,
    pub(crate) csrs: Csrs,
    pub(crate) mem: M,
}

impl Processor<FakeMemory> {
    pub fn get_mem(&mut self) -> &mut FakeMemory {
        &mut self.mem
    }
}

impl<M> Processor<M> {
    pub fn new(pc: u64, mem: M) -> Self {
        Processor {
            pc: pc,
            regs: Regs::new(),
            csrs: Csrs::new(),
            mem,
        }
    }

    pub fn mem(&mut self) -> &M {
        &self.mem
    }

    pub fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }

    #[inline(never)]
    pub fn step(&mut self, matchers: &[Matcher<M>])
    where
        M: Memory,
    {
        let insn = self.mem.read_w(self.pc);
        trace!("0x{:x} inst 0x{:x}", self.pc, insn);
        for matcher in matchers {
            if matcher.matches(insn) {
                matcher.exec(self, insn);
                return;
            }
        }
        panic!(format!("Unmatched instruction: 0x{:x}", insn));
    }

    #[inline(always)]
    pub fn advance_pc(&mut self) {
        self.pc += 4;
    }

    #[inline(always)]
    pub fn set_pc(&mut self, pc: u64) {
        info!("0x{:x} > Setting pc to 0x{:x}", self.pc, pc);
        self.pc = pc;
    }

    #[inline(always)]
    pub fn pc(&self) -> u64 {
        self.pc
    }
}
