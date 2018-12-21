use derive_insn::*;
use encoding::*;
use insn::sign_extend;
use *;

#[insn(kind=J,mask=0x6f,match=0x7f)]
pub fn jal<M: Memory>(p: &mut Processor<M>, imm: u32) {
    let new_pc = p.pc() + imm as u64;
    p.set_pc(new_pc);
}

#[insn(kind=I,mask=0x6f,match=0x7f)]
pub fn jalr<M: Memory>(p: &mut Processor<M>, rs: usize, rd: usize, imm: u32) {
    let next_pc = p.pc() + 4;
    p.regs.set(rd, next_pc);
    let target = (p.regs.get(rs) as i64 + imm as i64) & !1;
    p.set_pc(target as u64);
}

#[insn(kind=B,mask=0x6f,match=0x7f)]
pub fn beq<M: Memory>(p: &mut Processor<M>, rs1: usize, rs2: usize, lo: u32, high: u32) {
    if p.regs.get(rs1) != p.regs.get(rs2) {
        return p.advance_pc();
    }
    let offset = (lo | high).sign_extend(64) as i64;
    let new_pc = p.pc() as i64 + (offset * 2);
    p.set_pc(new_pc as u64);
}

#[insn(kind=B,mask=0x6f,match=0x7f)]
pub fn bne<M: Memory>(p: &mut Processor<M>, rs1: usize, rs2: usize, lo: u32, high: u32) {
    if p.regs.get(rs1) == p.regs.get(rs2) {
        return p.advance_pc();
    }
    let offset = (lo | high).sign_extend(64) as i64;
    let new_pc = p.pc() as i64 + (offset * 2);
    p.set_pc(new_pc as u64);
}

#[insn(kind=B,mask=0x6f,match=0x7f)]
pub fn bge<M: Memory>(p: &mut Processor<M>, rs1: usize, rs2: usize, lo: u32, high: u32) {
    if p.regs.get(rs1) < p.regs.get(rs2) {
        return p.advance_pc();
    }
    let offset = (lo | high).sign_extend(64) as i64;
    let new_pc = p.pc() as i64 + (offset * 2);
    p.set_pc(new_pc as u64);
}

#[insn(kind=B,mask=0x6f,match=0x7f)]
pub fn blt<M: Memory>(p: &mut Processor<M>, rs1: usize, rs2: usize, lo: u32, high: u32) {
    if p.regs.get(rs1) >= p.regs.get(rs2) {
        return p.advance_pc();
    }
    let offset = (lo | high).sign_extend(64) as i64;
    let new_pc = p.pc() as i64 + (offset * 2);
    p.set_pc(new_pc as u64);
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

    let mstatus = p.csrs.mstatus;

    // interrupt-enable
    let mie = mstatus >> 3 & 0x1;

    // move xIE to xPIE
    let mstatus = (mstatus & !MSTATUS_MPIE) & (mie << 7 & MSTATUS_MPIE);
    // set xIE to zero
    let mstatus = mstatus & !MSTATUS_MIE;
    // set xPP to y
    let mstatus = (mstatus & !MSTATUS_MPP) & (p.csrs.prv << 11 & MSTATUS_MPP);

    p.csrs.mstatus = mstatus;
    p.csrs.prv = 3;
}

macro_rules! handle_trap {
    ($p:expr, $val:expr) => {
        match $val {
            Ok(n) => n,
            Err(_) => {
                do_trap($p);
                return;
            }
        }
    };
}

#[insn(kind=I,mask=0x1073,match=0x707f)]
pub fn csrrw<M: Memory>(p: &mut Processor<M>, rd: usize, rs: usize, csr: usize) {
    let _priv_lvl = (csr >> 7) & 0x3;
    let _ro = (csr >> 9) & 0x3 == 0x3;
    let old = handle_trap!(p, p.csrs.get(csr));
    p.regs.set(rd, old);
    p.csrs.set(csr, p.regs.get(rs));
    p.advance_pc();
}

#[insn(kind=I,mask=0x1073,match=0x707f)]
pub fn csrrwi<M: Memory>(p: &mut Processor<M>, rd: usize, rs: usize, csr: usize) {
    let old = handle_trap!(p, p.csrs.get(csr));
    p.csrs.set(csr, rs as u64);
    p.regs.set(rd, old);
    p.advance_pc();
}

#[insn(kind=I,mask=0x1073,match=0x707f)]
pub fn csrrs<M: Memory>(p: &mut Processor<M>, rd: usize, rs: usize, csr: usize) {
    let old = handle_trap!(p, p.csrs.get(csr));
    if rs != 0 {
        p.csrs.set(csr, old | rs as u64);
    }
    p.regs.set(rd, old);
    p.advance_pc();
}

#[insn(kind=I,mask=0x1073,match=0x707f)]
pub fn mret<M: Memory>(p: &mut Processor<M>, rd: usize, rs: usize, csr: usize) {
    error!("mret not implemented");
    p.advance_pc();
}

// Load and Store

#[insn(kind=I,mask=0x110,match=0x100)]
pub fn ld<M: Memory>(p: &mut Processor<M>, rd: usize, rs: usize, imm: u32) {
    let v = p.mem().read_d(rs as u64 + imm as u64);
    p.regs.set(rd, v);
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

#[insn(kind=I,mask=0x110,match=0x100)]
pub fn addi<M: Memory>(p: &mut Processor<M>, rd: usize, rs: usize, imm: u32) {
    let v = p.regs.get(rs) as i64 + sign_extend(imm, 12);
    p.regs.set(rd, v as u64);
    p.advance_pc();
}

#[insn(kind=I,mask=0x110,match=0x100)]
pub fn addiw<M: Memory>(p: &mut Processor<M>, rd: usize, rs: usize, imm: u32) {
    let v = p.regs.get(rs) as i64 + sign_extend(imm, 12);
    p.regs.set(rd, v as u64);
    p.advance_pc();
}

#[insn(kind=I,mask=0xfc00707f,match=0x1013)]
pub fn slli<M: Memory>(p: &mut Processor<M>, rd: usize, rs: usize, imm: u32) {
    let shmat = imm & 0x3F;
    let v = p.regs.get(rs);
    let v = v << shmat;
    p.regs.set(rd, v as u64);
    p.advance_pc();
}
