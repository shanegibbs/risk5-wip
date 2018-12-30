use derive_insn::*;
use insn::sign_extend;
use *;
use itypes::*;

pub fn jal<M: Memory>(p: &mut Processor<M>, i: Jtype) {
    let new_pc = p.pc() as i64 + i.imm();
    p.set_pc(new_pc as u64);
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
    if p.regs.get(i.rs1() as usize) < p.regs.get(i.rs2() as usize) {
        return p.advance_pc();
    }
    i.jump(p);
}

pub fn blt<M: Memory>(p: &mut Processor<M>, i: Btype) {
    if p.regs.get(i.rs1() as usize) >= p.regs.get(i.rs2() as usize) {
        return p.advance_pc();
    }
    i.jump(p);
}

#[insn(kind=U,mask=0x110,match=0x100)]
pub fn auipc<M: Memory>(p: &mut Processor<M>, rd: usize, imm: u32) {
    let offset = sign_extend(imm, 32) as i64;
    let v = p.pc() as i64 + offset;
    p.regs.set(rd, v as u64);
    p.advance_pc();
}


#[insn(kind=U,mask=0x110,match=0x100)]
pub fn lui<M: Memory>(p: &mut Processor<M>, rd: usize, imm: u32) {
    let v = insn::sign_extend(imm, 32) as i64;
    p.regs.set(rd, v as u64);
    p.advance_pc();
}

pub fn do_trap<M: Memory>(p: &mut Processor<M>) {
    debug!("Doing trap");
    if p.csrs.prv != 3 {
        panic!("Unimplemented prv level");
    }

    let mtvec = p.csrs.mtvec;

    let mode = mtvec & 0x3;
    if mode != 0 {
        panic!("Unimplemented mtvec mode");
    }

    p.csrs.mepc = p.pc();
    p.set_pc(mtvec & !0x3);

    {
        let m = &mut p.csrs.mstatus;

        // move xIE to xPIE and set xIE to 0
        m.move_machine_interrupt_enabled_to_prior();
        m.set_machine_interrupt_enabled(0);
        // set xPP to prv
        m.set_machine_previous_privilege(p.csrs.prv);

    }

    p.csrs.prv = 3;
}

macro_rules! handle_trap {
    ($p:expr, $val:expr) => {
        match $val {
            Ok(n) => n,
            Err(cause) => {
                $p.csrs.mcause = cause;
                do_trap($p);
                return;
            }
        }
    };
}

pub fn csrrw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let old = handle_trap!(p, p.csrs.get(i.u_imm() as usize));
    p.regs.set(i.rd() as usize, old);
    p.csrs.set(i.u_imm() as usize, p.regs.get(i.rs1() as usize));
    p.advance_pc();
}

pub fn csrrwi<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let old = handle_trap!(p, p.csrs.get(i.u_imm() as usize));
    p.csrs.set(i.u_imm() as usize, i.rs1() as u64);
    p.regs.set(i.rd() as usize, old);
    p.advance_pc();
}

pub fn csrrs<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let old = handle_trap!(p, p.csrs.get(i.u_imm() as usize));
    if i.rs1() != 0 {
        let rs1 = p.regs.get(i.rs1() as usize);
        p.csrs.set(i.u_imm() as usize, old | rs1);
    }
    p.regs.set(i.rd() as usize, old);
    p.advance_pc();
}

pub fn mret<M: Memory>(p: &mut Processor<M>, _: Itype) {
    let mpie = p.csrs.mstatus.machine_prior_interrupt_enabled();

    p.csrs.mstatus.set_machine_interrupt_enabled(mpie);
    p.csrs.mstatus.set_machine_prior_interrupt_enabled(1);
    p.csrs.mstatus.set_machine_previous_privilege(0);

    p.advance_pc();
}

pub fn ecall<M: Memory>(p: &mut Processor<M>, _: Itype) {
    if p.csrs.prv != 3 {
        panic!("Unimplemented prv level");
    }
    p.csrs.mcause = 0xb; // env call from M mode
    do_trap(p)
}

// Load and Store

pub fn ld<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let v = p.mem().read_d((i.rs1() as i64 + i.imm()) as u64);
    p.regs.set(i.rd() as usize, v);
    p.advance_pc();
}

#[insn(kind=S,mask=0x110,match=0x100)]
pub fn sw<M: Memory>(p: &mut Processor<M>, rs1: usize, rs2: usize, imm: i32) {
    let rv1 = p.regs.get(rs1) as i64;
    let rv2 = p.regs.get(rs2) as i64;
    let offset = rv1 + imm as i64;
    p.mem_mut().write_w(offset as u64, rv2 as u32);

    p.advance_pc();
}

// Integer Computational Instructions

pub fn addi<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let v = p.regs.geti(i.rs1() as usize).wrapping_add(i.imm());
    p.regs.seti(i.rd() as usize, v);
    p.advance_pc();
}

pub fn addiw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let a = (p.regs.get(i.rs1() as usize) as i64).wrapping_add(i.imm());
    let b = a << 32 >> 32;
    p.regs.set(i.rd() as usize, b as u64);
    p.advance_pc();
}

pub fn slli<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let shmat = i.imm() & 0x3F;
    let v = p.regs.get(i.rs1() as usize);
    let v = v << shmat;
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}
