pub use self::amo::*;
pub use self::csr::*;
use crate::itypes::*;
use crate::*;

mod amo;
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
            let pprv = p.prv();
            let m = &mut p.csrs_mut().mstatus;

            // move xIE to xPIE and set xIE to 0
            m.move_machine_interrupt_enabled_to_prior();
            m.set_machine_interrupt_enabled(0);
            // set xPP to prv
            m.set_machine_previous_privilege(pprv);
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

pub fn ecall<M: Memory>(p: &mut Processor<M>, _: Itype) {
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

// Integer Computational Instructions

pub fn add<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p
        .regs
        .geti(i.rs1() as usize)
        .wrapping_add(p.regs.geti(i.rs2() as usize));
    target(p, i, v as u64);
}

pub fn addw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = (p.regs.geti(i.rs1() as usize) as i32) + (p.regs.geti(i.rs2() as usize) as i32);
    target(p, i, v as u64);
}

pub fn addi<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let v = p.regs.geti(i.rs1() as usize).wrapping_add(i.imm());
    target(p, i, v as u64);
}

pub fn addiw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let a = (p.regs.get(i.rs1() as usize) as i64).wrapping_add(i.imm());
    let b = a << 32 >> 32;
    target(p, i, b as u64);
}

fn target<M: Memory, I: FieldRd>(p: &mut Processor<M>, i: I, v: u64) {
    p.regs.set(i.rd() as usize, v);
    p.advance_pc();
}

pub fn and<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p.regs.geti(i.rs1() as usize) & p.regs.geti(i.rs2() as usize);
    p.regs.seti(i.rd() as usize, v);
    p.advance_pc();
}

pub fn sub<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p.regs.geti(i.rs1() as usize) as i64 - p.regs.geti(i.rs2() as usize) as i64;
    p.regs.seti(i.rd() as usize, v);
    p.advance_pc();
}

pub fn or<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p.regs.geti(i.rs1() as usize) | p.regs.geti(i.rs2() as usize);
    p.regs.seti(i.rd() as usize, v);
    p.advance_pc();
}

pub fn xor<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p.regs.geti(i.rs1() as usize) ^ p.regs.geti(i.rs2() as usize);
    p.regs.seti(i.rd() as usize, v);
    p.advance_pc();
}

pub fn sltiu<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.get(i.rs1() as usize);
    p.regs
        .seti(i.rd() as usize, if rs1 < i.immu() { 1 } else { 0 });
    p.advance_pc();
}

pub fn sltu<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let rs1 = p.regs.get(i.rs1() as usize);
    let rs2 = p.regs.get(i.rs2() as usize);
    p.regs.set(i.rd() as usize, if rs1 < rs2 { 1 } else { 0 });
    p.advance_pc();
}

pub fn andi<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let v = p.regs.geti(i.rs1() as usize) & i.imm();
    p.regs.seti(i.rd() as usize, v);
    p.advance_pc();
}

pub fn ori<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let v = p.regs.geti(i.rs1() as usize) | i.imm();
    p.regs.seti(i.rd() as usize, v);
    p.advance_pc();
}

pub fn xori<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let v = p.regs.geti(i.rs1() as usize) ^ i.imm();
    p.regs.seti(i.rd() as usize, v);
    p.advance_pc();
}

pub fn subw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let a = (p.regs.get(i.rs1() as usize) as i64) - (p.regs.get(i.rs2() as usize) as i64);
    let b = a << 32 >> 32;
    p.regs.set(i.rd() as usize, b as u64);
    p.advance_pc();
}

pub fn sll<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let shmat = p.regs.get(i.rs2() as usize) & 0x3F;
    let v = p.regs.get(i.rs1() as usize);
    let v = v << shmat;
    p.regs.set(i.rd() as usize, v);
    p.advance_pc();
}

pub fn sllw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let shmat = p.regs.get(i.rs2() as usize) & 0x1F;
    let v = p.regs.get(i.rs1() as usize) as u32;
    let v = v << shmat;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn slli<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let shmat = i.imm() & 0x3F;
    let v = p.regs.get(i.rs1() as usize);
    let v = v << shmat;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn slliw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let shmat = i.imm() & ((2 << 5) - 1);
    let v = p.regs.get(i.rs1() as usize) as u32;
    let v = (((v << shmat) as i32) as i64) << 32 >> 32;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn srl<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let shmat = p.regs.get(i.rs2() as usize) & 0x3f;
    let v = p.regs.get(i.rs1() as usize);
    let v = v >> shmat;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn srli<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let shmat = i.imm() & 0x3F;
    let v = p.regs.get(i.rs1() as usize);
    let v = v >> shmat;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn srliw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let shmat = i.imm() & ((2 << 5) - 1);
    let v = p.regs.get(i.rs1() as usize) as u32;
    let v = v >> shmat;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn sra<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let shmat = p.regs.get(i.rs2() as usize) & 0x3f;
    let v = p.regs.get(i.rs1() as usize) as i64;
    let v = v >> shmat;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn sraw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let shmat = p.regs.get(i.rs2() as usize) & 0x3f;
    let v = p.regs.get(i.rs1() as usize) as i32;
    let v = v >> shmat;
    let sign_extended = ((v as i64) << 32 >> 32) as u64;
    p.regs.set(i.rd() as usize, sign_extended);
    p.advance_pc();
}

pub fn srai<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let shmat = i.imm() & 0x3F;
    let v = p.regs.get(i.rs1() as usize) as i64;
    let v = v >> shmat;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn sraiw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let shmat = i.imm() & 0x3F;
    let v = p.regs.get(i.rs1() as usize) as i32;
    let v = v >> shmat;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn mul<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p
        .regs
        .get(i.rs1() as usize)
        .wrapping_mul(p.regs.get(i.rs2() as usize));
    p.regs.set(i.rd() as usize, v);
    p.advance_pc();
}

pub fn mulw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p.regs.get(i.rs1() as usize) * p.regs.get(i.rs2() as usize);
    p.regs.set(i.rd() as usize, (v as u32) as u64);
    p.advance_pc();
}

pub fn div<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let rhs = p.regs.geti(i.rs2() as usize);
    let v = if rhs == 0 {
        i64::max_value()
    } else {
        p.regs.geti(i.rs1() as usize) / rhs
    };
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn divu<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let rhs = p.regs.get(i.rs2() as usize);
    let v = if rhs == 0 {
        u64::max_value()
    } else {
        p.regs.get(i.rs1() as usize) / rhs
    };
    p.regs.set(i.rd() as usize, v);
    p.advance_pc();
}

pub fn divw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let rhs = p.regs.geti(i.rs2() as usize);
    let v = if rhs == 0 {
        i64::max_value()
    } else {
        p.regs.geti(i.rs1() as usize) / rhs
    };
    p.regs.set(i.rd() as usize, (v as u32) as u64);
    p.advance_pc();
}

pub fn divuw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let rhs = p.regs.get(i.rs2() as usize);
    let v = if rhs == 0 {
        u64::max_value()
    } else {
        p.regs.get(i.rs1() as usize) / rhs
    };
    p.regs.set(i.rd() as usize, (v as u32) as u64);
    p.advance_pc();
}

pub fn rem<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p.regs.geti(i.rs1() as usize) % p.regs.geti(i.rs2() as usize);
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn remu<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p.regs.get(i.rs1() as usize) % p.regs.get(i.rs2() as usize);
    p.regs.set(i.rd() as usize, v);
    p.advance_pc();
}

pub fn remw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p.regs.geti(i.rs1() as usize) % p.regs.geti(i.rs2() as usize);
    p.regs.set(i.rd() as usize, (v as u32) as u64);
    p.advance_pc();
}

pub fn remuw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p.regs.get(i.rs1() as usize) % p.regs.get(i.rs2() as usize);
    p.regs.set(i.rd() as usize, (v as u32) as u64);
    p.advance_pc();
}
