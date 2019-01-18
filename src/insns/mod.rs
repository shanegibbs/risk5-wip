use crate::itypes::*;
use crate::*;

pub struct Trap {
    cause: u64,
    value: u64,
}

impl Trap {
    pub fn illegal_insn() -> Trap {
        Trap { cause: 2, value: 0 }
    }
}

// macro_rules! handle_trap {
//     ($p:expr, $trap:expr) => {
//         match $trap {
//             (Ok(n), _) => n,
//             (Err(trap), 3) => {
//                 $p.csrs_mut().mcause = trap.cause;
//                 $p.csrs_mut().mtval = trap.value;
//                 do_trap($p, trap.cause, trap.value);
//                 return;
//             }
//             (Err(_trap), _prv) => {
//                 panic!("Unimplemented trap level");
//             }
//         }
//     };
// }

pub mod mem;
pub mod prv;

pub mod csr;
pub use self::csr::*;

mod amo;
pub use self::amo::*;

pub fn jal<M: Memory>(p: &mut Processor<M>, i: Jtype) {
    let old_pc = p.pc();
    let new_pc = (old_pc as i64 + i.imm()) as u64;
    p.set_pc(new_pc);
    p.regs.set(i.rd() as usize, old_pc + 4);
}

pub fn jalr<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let next_pc = p.pc() + 4;
    p.regs.set(i.rd() as usize, next_pc);
    let target = (p.regs.get(i.rs1() as usize) as i64 + i.imm()) & !1;
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
    let prv = p.csrs().prv;
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
        error!("here");
        csrs.mstatus.set_supervisor_previous_privilege(1);

        csrs.prv = 1;
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
            let pprv = p.csrs().prv;
            let m = &mut p.csrs_mut().mstatus;

            // move xIE to xPIE and set xIE to 0
            m.move_machine_interrupt_enabled_to_prior();
            m.set_machine_interrupt_enabled(0);
            // set xPP to prv
            m.set_machine_previous_privilege(pprv);
        }

        p.csrs_mut().prv = 3;
    }
}

pub fn mret<M: Memory>(p: &mut Processor<M>, _: Itype) {
    let pprv = p.csrs().mstatus.machine_previous_privilege();
    let pie = p.csrs().mstatus.machine_prior_interrupt_enabled();
    let epc = p.csrs().mepc;

    p.csrs_mut().mstatus.set_machine_interrupt_enabled(pie);
    p.csrs_mut().mstatus.set_machine_prior_interrupt_enabled(1);
    p.csrs_mut().mstatus.set_machine_previous_privilege(0);

    p.csrs_mut().prv = pprv;
    p.set_pc(epc);
}

pub fn ecall<M: Memory>(p: &mut Processor<M>, _: Itype) {
    if p.csrs().prv != 3 {
        panic!("Unimplemented prv level");
    }
    do_trap(p, 0xb, 0)
}

// Integer Computational Instructions

pub fn add<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v = p
        .regs
        .geti(i.rs1() as usize)
        .wrapping_add(p.regs.geti(i.rs2() as usize));
    p.regs.seti(i.rd() as usize, v);
    p.advance_pc();
}

pub fn addw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let v =
        (p.regs.geti(i.rs1() as usize) as i32).wrapping_add(p.regs.geti(i.rs2() as usize) as i32);
    p.regs.set(i.rd() as usize, v as u64);
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

pub fn addi<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let v = p.regs.geti(i.rs1() as usize).wrapping_add(i.imm());
    p.regs.seti(i.rd() as usize, v);
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

pub fn addiw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let a = (p.regs.get(i.rs1() as usize) as i64).wrapping_add(i.imm());
    let b = a << 32 >> 32;
    p.regs.set(i.rd() as usize, b as u64);
    p.advance_pc();
}

pub fn subw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let a = (p.regs.get(i.rs1() as usize) as i64) - (p.regs.get(i.rs2() as usize) as i64);
    let b = a << 32 >> 32;
    p.regs.set(i.rd() as usize, b as u64);
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
