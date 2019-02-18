pub use self::amo::*;
pub use self::csr::*;
use crate::itypes::*;
use crate::*;

mod amo;
pub mod comp;
pub mod csr;
pub mod mem;
pub mod prv;

pub struct Trap {
    cause: u64,
    value: u64,
}

impl Trap {
    pub fn illegal_insn() -> Trap {
        Trap { cause: 2, value: 0 }
    }
}

pub fn jal<M: Memory>(p: &mut Processor<M>, i: Jtype) {
    let old_pc = p.pc();
    let new_pc = (old_pc as i64 + i.imm()) as u64;
    p.set_pc(new_pc);
    p.regs.set(i.rd() as usize, old_pc + 4);
}

pub fn jalr<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let next_pc = p.pc() + 4;
    let target = (p.regs.get(i.rs1() as usize) as i64 + i.imm()) & !1;
    p.regs.set(i.rd() as usize, next_pc);
    p.set_pc(target as u64);
}

pub fn beq<M: Memory>(p: &mut Processor<M>, i: Btype) {
    if p.regs.get(i.rs1() as usize) != p.regs.get(i.rs2() as usize) {
        return p.advance_pc();
    }
    i.jump(p);
}

pub fn bne<M: Memory>(p: &mut Processor<M>, i: Btype) {
    if p.regs.get(i.rs1() as usize) == p.regs.get(i.rs2() as usize) {
        return p.advance_pc();
    }
    i.jump(p);
}

pub fn bge<M: Memory>(p: &mut Processor<M>, i: Btype) {
    if (p.regs.get(i.rs1() as usize) as i64) < (p.regs.get(i.rs2() as usize) as i64) {
        return p.advance_pc();
    }
    i.jump(p);
}

pub fn blt<M: Memory>(p: &mut Processor<M>, i: Btype) {
    if (p.regs.get(i.rs1() as usize) as i64) >= (p.regs.get(i.rs2() as usize) as i64) {
        return p.advance_pc();
    }
    i.jump(p);
}

pub fn bgeu<M: Memory>(p: &mut Processor<M>, i: Btype) {
    if p.regs.get(i.rs1() as usize) < p.regs.get(i.rs2() as usize) {
        return p.advance_pc();
    }
    i.jump(p);
}

pub fn bltu<M: Memory>(p: &mut Processor<M>, i: Btype) {
    if p.regs.get(i.rs1() as usize) >= p.regs.get(i.rs2() as usize) {
        return p.advance_pc();
    }
    i.jump(p);
}

pub fn auipc<M: Memory>(p: &mut Processor<M>, i: Utype) {
    let v = p.pc() as i64 + i.imm();
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn lui<M: Memory>(p: &mut Processor<M>, i: Utype) {
    p.regs.set(i.rd() as usize, i.imm() as u64);
    p.advance_pc();
}

/*
 *
 * PRV
 * ---
 * 0 User
 * 1 Supervisor
 * 3 Machine
 *
 */
pub fn do_trap<M: Memory>(p: &mut Processor<M>, cause: u64, val: u64) {
    let prv = p.csrs().prv();
    let medeleg = p.csrs().medeleg;

    debug!("Doing trap prv={} cause=0x{:x} value={:x}", prv, cause, val);

    if prv <= 1 && ((medeleg >> cause) & 0x1) == 1 {
        // handle in supervisor mode if in supervisor or user mode
        // or if the bit from cause is set is set in medeleg

        let pc = p.pc();
        let stvec = p.csrs().stvec;

        p.set_pc(stvec);

        let mut csrs = p.csrs_mut();
        csrs.scause = cause;
        csrs.sepc = pc;
        csrs.stval = val;

        // move xIE to xPIE and set xIE to 0
        csrs.mstatus.move_supervisor_interrupt_enabled_to_prior();
        csrs.mstatus.set_supervisor_interrupt_enabled(0);
        // set xPP to prv
        csrs.mstatus.set_supervisor_previous_privilege(1);

        csrs.mstatus.set_supervisor_previous_privilege(prv);
        p.set_prv(1);
    } else {
        // handle in machine mode by default

        p.csrs_mut().mcause = cause;
        p.csrs_mut().mtval = val;

        trace!("medeleg 0x{0:016x} b'{0:064b}", p.csrs().medeleg);
        trace!("mtvec   0x{0:016x} b'{0:064b}", p.csrs().mtvec);
        // trace!("stvec   0x{0:016x} b'{0:064b}", p.csrs().stvec);

        let mtvec = p.csrs().mtvec;

        let mode = mtvec & 0x3;
        if mode != 0 {
            panic!("Unimplemented mtvec mode");
        }

        p.csrs_mut().mepc = p.pc();
        p.set_pc(mtvec & !0x1);

        {
            let m = &mut p.csrs_mut().mstatus;

            // move xIE to xPIE and set xIE to 0
            m.move_machine_interrupt_enabled_to_prior();
            m.set_machine_interrupt_enabled(0);
            // set xPP to prv
            m.set_machine_previous_privilege(prv);
        }

        p.set_prv(3);
    }
}

pub fn mret<M: Memory>(p: &mut Processor<M>, _: Itype) {
    let pprv = p.csrs().mstatus.machine_previous_privilege();
    let pie = p.csrs().mstatus.machine_prior_interrupt_enabled();
    let epc = p.csrs().mepc;

    p.csrs_mut().mstatus.set_machine_interrupt_enabled(pie);
    p.csrs_mut().mstatus.set_machine_prior_interrupt_enabled(1);
    p.csrs_mut().mstatus.set_machine_previous_privilege(0);

    p.set_prv(pprv);
    p.set_pc(epc);
}

pub fn sret<M: Memory>(p: &mut Processor<M>, _: Itype) {
    let pprv = p.csrs().mstatus.supervisor_previous_privilege();
    let pie = p.csrs().mstatus.supervisor_prior_interrupt_enabled();
    let spc = p.csrs().sepc;

    p.csrs_mut().mstatus.set_supervisor_interrupt_enabled(pie);
    p.csrs_mut()
        .mstatus
        .set_supervisor_prior_interrupt_enabled(1);
    p.csrs_mut().mstatus.set_supervisor_previous_privilege(0);

    p.set_prv(pprv);
    p.set_pc(spc);
}

pub fn ecall<M: Memory>(p: &mut Processor<M>, _: Itype) {
    let a7 = p.regs.get(17 as usize);
    if a7 <= 8 {
        use std::io::{self, Write};

        let a0 = p.regs.get(10 as usize);
        if a7 == 1 {
            trace!("ecall 0x{:x} {}", a7, a0 as u8 as char);
            write!(io::stderr(), "{}", a0 as u8 as char);
        } else {
            warn!("ecall 0x{:x} {}", a7, a0);
        }
    }
    let prv = p.csrs().prv();
    do_trap(
        p,
        match prv {
            0 => 8,
            1 => 9,
            3 => 11,
            prv => panic!("Unimplemented prv level for ecall {}", prv),
        },
        0,
    )
}
