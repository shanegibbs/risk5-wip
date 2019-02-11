use crate::matcher::{Matcher, Matchers};
use crate::Mmu;
use crate::{Memory, Regs};
use csrs::{Csrs, PostSetOp, SetMemMode};

mod csrs;

#[derive(Debug)]
pub struct Processor<M> {
    pc: u64,
    csrs: Csrs,
    pub(crate) regs: Regs,
    mmu: Mmu<M>,
    pub(crate) trigger: bool,
}

impl<M> Processor<M> {
    pub fn new(mem: M) -> Self {
        Processor {
            pc: 0x1000,
            csrs: Csrs::new(),
            regs: Regs::new(),
            mmu: Mmu::new(mem),
            trigger: false,
        }
    }

    pub fn prv(&self) -> u64 {
        self.csrs.prv()
    }

    pub fn set_prv(&mut self, prv: u64) {
        debug!("Setting prv to {}", prv);
        self.csrs.set_prv(prv);
        self.mmu.set_prv(prv, &self.csrs.mstatus);
    }

    pub fn set_mem_mode(&mut self, op: SetMemMode) {
        if op.mode == 0 {
            self.mmu.set_bare_mode();
        } else if op.mode == 8 {
            self.mmu.set_page_mode(op.asid as u16, op.ppn);
        } else {
            panic!("Unsupported memory mode")
        }
    }

    pub fn get_reg(&self, i: u32) -> u64 {
        self.regs.get(i as usize)
    }

    pub fn set_csr(&mut self, i: u32, val: u64) {
        match self.csrs.set(i as usize, val) {
            PostSetOp::None => (),
            PostSetOp::SetMemMode(m) => self.set_mem_mode(m),
            PostSetOp::UpdateMmuPrv => self.mmu.set_prv(self.csrs.prv(), &self.csrs.mstatus),
        }
    }

    pub fn csrs(&mut self) -> &Csrs {
        &self.csrs
    }

    pub fn csrs_mut(&mut self) -> &mut Csrs {
        &mut self.csrs
    }

    pub(crate) fn mmu(&self) -> &Mmu<M> {
        &self.mmu
    }

    pub(crate) fn mmu_mut(&mut self) -> &mut Mmu<M> {
        &mut self.mmu
    }

    fn execute(&mut self, insn: u32, matcher: &Matcher<M>)
    where
        M: Memory,
    {
        matcher.exec(self, insn);
    }

    #[inline(never)]
    pub fn step(&mut self, matchers: &mut Matchers<M>)
    where
        M: Memory,
    {
        let insn = match self.mmu.read_insn(self.pc) {
            Ok(insn) => insn,
            Err(()) => {
                crate::insns::do_trap(self, 12, self.pc);
                return;
            }
        };
        trace!("0x{:x} inst 0x{:x}", self.pc, insn);

        let matcher = matchers.find_for(insn);
        self.execute(insn, matcher);

        // for (i, matcher) in matchers.enumerate() {
        //     if matcher.matches(insn) {
        //         self.execute(insn, matcher);
        //         // matcher.exec(self, insn);
        //         return i;
        //     }
        // }
        // error!("Unmatched instruction: 0x{:x}", insn);
        // panic!("Unmatched instruction");
    }

    pub fn advance_pc(&mut self) {
        self.pc += 4;
    }

    pub fn set_pc(&mut self, pc: u64) {
        if self.trigger {
            warn!("0x{:x} > Setting pc to 0x{:x}", self.pc, pc);
        } else {
            info!("0x{:x} > Setting pc to 0x{:x}", self.pc, pc);
        }
        self.pc = pc;
    }

    pub fn pc(&self) -> u64 {
        self.pc
    }
}

use crate::logrunner::RestorableState;
impl<'a, M> Into<Processor<M>> for RestorableState<'a, M> {
    fn into(self) -> Processor<M> {
        let RestorableState { state, memory } = self;
        Processor {
            pc: state.pc,
            csrs: state.into(),
            regs: state.xregs.into(),
            mmu: RestorableState { state, memory }.into(),
            trigger: false,
        }
    }
}

use crate::logrunner::State;
impl<M> Into<State> for &Processor<M> {
    fn into(self) -> State {
        State {
            id: 0,
            pc: self.pc,
            prv: self.csrs.prv(),
            mstatus: self.csrs.mstatus.val(),
            mepc: self.csrs.mepc,
            mtvec: self.csrs.mtvec,
            mcause: self.csrs.mcause,
            mscratch: self.csrs.mscratch,
            minstret: 0, // self.csrs.minstret,
            mie: (&self.csrs.mie).into(),
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
