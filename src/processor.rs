use crate::csrs::Csrs;
use crate::Mmu;
use crate::{Matcher, Memory, Regs};

#[derive(Debug)]
pub struct Processor<M> {
    pc: u64,
    pub(crate) regs: Regs,
    pub(crate) csrs: Csrs,
    pub(crate) mmu: Mmu<M>,
}

impl<M> Processor<M> {
    pub fn new(pc: u64, mem: M) -> Self {
        Processor {
            pc: pc,
            regs: Regs::new(),
            csrs: Csrs::new(),
            mmu: Mmu::new(mem),
        }
    }

    pub fn mmu(&mut self) -> &Mmu<M> {
        &self.mmu
    }

    pub fn mmu_mut(&mut self) -> &mut Mmu<M> {
        &mut self.mmu
    }

    #[inline(never)]
    pub fn step(&mut self, matchers: &[Matcher<M>])
    where
        M: Memory,
    {
        let insn = self.mmu.read_w(self.pc);
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
