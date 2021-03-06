use crate::bitfield::{Interrupt, Mstatus};
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
    insn_counter: u64,
    timer: u64,
    ecall_counter: usize,
}

impl<M> Processor<M> {
    pub fn new(mem: M) -> Self {
        Processor {
            pc: 0x1000,
            csrs: Csrs::new(),
            regs: Regs::new(),
            mmu: Mmu::new(mem),
            trigger: false,
            insn_counter: 0,
            timer: u64::max_value(),
            ecall_counter: 0,
        }
    }

    pub fn getchar(&mut self) -> u64 {
        let counter = self.ecall_counter;
        self.ecall_counter += 1;

        let cmd = "uname -a; cat /proc/cpuinfo; cat /proc/meminfo";
        let n: Vec<_> = cmd.chars().map(|i| i as u64).collect();

        if counter == n.len() {
            return 13;
        }
        if counter > n.len() {
            return -1 as isize as u64;
        }
        n[counter]
    }

    pub fn insn_counter(&self) -> u64 {
        self.insn_counter
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

    pub(crate) fn machine_status(&self) -> &Mstatus {
        &self.csrs.mstatus
    }

    pub(crate) fn pending_interrupts(&self) -> Interrupt {
        (self.csrs.mip.val() & self.csrs.mie.val()).into()
    }

    pub fn set_timer(&mut self, delta: u64) {
        self.timer = self.insn_counter + delta;
        debug!("Setting timer for {} ({})", delta, self.timer);
        if self.prv() != 1 {
            // TODO need different timers for different prv?
            unimplemented!("set timer for {}", self.prv());
        }
    }

    pub fn check_clock(&mut self) {
        if self.insn_counter > self.timer {
            self.timer = u64::max_value();
            self.csrs.mip.set_supervisor_timer_interrupt(1);
            // error!("mip {:?}", self.csrs.mip);
            // error!("mie {:?}", self.csrs.mie);
            // error!("mideleg: 0x{:x}", self.csrs().mideleg);
            // error!("mstatus: {:?}", self.machine_status());
        }
    }

    pub fn handle_interrupt(&mut self)
    where
        M: Memory,
    {
        // let interrupts = self.pending_interrupts();
        // if interrupts.val() > 0 {
        //     warn!("should handle {:?}", interrupts);
        //     crate::insns::do_trap(self, 0x8000_0000_0000_0007, self.pc);
        // }
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

        // if self.pc == 0x132088 && insn == 0x00843783 {
        //     error!("here");
        // }

        let matcher = matchers.find_for(insn);
        self.execute(insn, matcher);

        self.insn_counter += 1;

        if self.insn_counter % 5000 == 0 {
            self.check_clock();
            self.handle_interrupt();
        }

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
        if pc == 0 {
            panic!("setting pc = 0 at 0x{:x}", self.pc());
        }

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
            timer: 0,        // TODO: store timer in state
            insn_counter: 0, // TODO: store insn_counter in state
            ecall_counter: 0,
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
            mip: (&self.csrs.mip).into(),
            medeleg: self.csrs.medeleg,
            mideleg: self.csrs.mideleg,
            mcounteren: self.csrs.mcounteren,
            scounteren: self.csrs.scounteren,
            sepc: self.csrs.sepc(),
            stval: self.csrs.stval,
            sscratch: self.csrs.sscratch,
            stvec: self.csrs.stvec,
            satp: (&self.csrs.satp).into(),
            scause: self.csrs.scause,
            xregs: (&self.regs).into(),
        }
    }
}
