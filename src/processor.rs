use crate::csrs::{Csrs, PostSetOp, SetMemMode};
use crate::Mmu;
use crate::{Matcher, Memory, Regs};

#[derive(Debug)]
pub struct Processor<M> {
    pc: u64,
    csrs: Csrs,
    pub(crate) regs: Regs,
    mmu: Mmu<M>,
}

impl<M> Processor<M> {
    pub fn new(pc: u64, mem: M) -> Self {
        Processor {
            pc: pc,
            csrs: Csrs::new(),
            regs: Regs::new(),
            mmu: Mmu::new(mem),
        }
    }

    #[inline(always)]
    pub fn set_mem_mode(&mut self, op: SetMemMode) {
        if op.mode == 0 {
            self.mmu.set_bare_mode();
        } else if op.mode == 8 {
            self.mmu.set_page_mode(op.asid as u16, op.ppn);
        } else {
            panic!("Unsupported memory mode")
        }
    }

    #[inline(always)]
    pub fn get_reg(&self, i: u32) -> u64 {
        self.regs.get(i as usize)
    }

    #[inline(always)]
    pub fn set_csr(&mut self, i: u32, val: u64) {
        match self.csrs.set(i as usize, val) {
            PostSetOp::None => (),
            PostSetOp::SetMemMode(m) => self.set_mem_mode(m),
        }
    }

    #[inline(always)]
    pub fn csrs(&mut self) -> &Csrs {
        &self.csrs
    }

    #[inline(always)]
    pub fn csrs_mut(&mut self) -> &mut Csrs {
        &mut self.csrs
    }

    #[inline(always)]
    pub fn mmu(&self) -> &Mmu<M> {
        &self.mmu
    }

    #[inline(always)]
    pub fn mmu_mut(&mut self) -> &mut Mmu<M> {
        &mut self.mmu
    }

    #[inline(never)]
    pub fn step(&mut self, matchers: &[Matcher<M>])
    where
        M: Memory,
    {
        let insn = match self.mmu.read_w(self.pc) {
            Ok(insn) => insn,
            Err(()) => {
                crate::insns::do_trap(self, 12, self.pc);
                return;
            }
        };
        trace!("0x{:x} inst 0x{:x}", self.pc, insn);
        for matcher in matchers {
            if matcher.matches(insn) {
                matcher.exec(self, insn);
                return;
            }
        }
        error!("Unmatched instruction: 0x{:x}", insn);
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

use crate::logrunner::State;
impl<M> Into<Processor<M>> for &State {
    fn into(self) -> Processor<M> {
        Processor {
            pc: self.pc,
            csrs: self.into(),
            regs: (self.xregs.as_slice()).into(),
            mmu: self.into(),
        }
    }
}

impl<M> Into<State> for &Processor<M> {
    fn into(self) -> State {
        State {
            id: 0,
            pc: self.pc,
            prv: self.csrs.prv,
            mstatus: self.csrs.mstatus.val(),
            mepc: self.csrs.mepc,
            mtvec: self.csrs.mtvec,
            mcause: self.csrs.mcause,
            mscratch: self.csrs.mscratch,
            minstret: 0, // self.csrs.minstret,
            mie: self.csrs.mie,
            mip: self.csrs.mip,
            medeleg: self.csrs.medeleg,
            mideleg: self.csrs.mideleg,
            mcounteren: self.csrs.mcounteren,
            scounteren: self.csrs.scounteren,
            sepc: self.csrs.sepc,
            stval: self.csrs.stval,
            sscratch: self.csrs.sscratch,
            stvec: self.csrs.stvec,
            satp: (&self.csrs.satp).into(),
            scause: self.csrs.scause,
            xregs: (&self.regs).into(),
        }
    }
}
