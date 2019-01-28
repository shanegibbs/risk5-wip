use super::*;

// Load and Store

macro_rules! mem {
    ($p:expr, $func:ident, $addr:expr) => {
        match $p.mmu().$func($addr) {
            Ok(v) => v,
            Err(_) => {
                do_trap($p, 13, $addr);
                return;
            }
        }
    };
    ($p:expr, $func:ident, $addr:expr, $val:expr) => {
        match $p.mmu_mut().$func($addr, $val) {
            Ok(v) => v,
            Err(_) => {
                do_trap($p, 15, $addr);
                return;
            }
        }
    };
}

pub fn lbu<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let addr = (rs1 + i.imm()) as u64;
    let v = mem!(p, read_b, addr);
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn lhu<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let addr = (rs1 + i.imm()) as u64;
    let v = mem!(p, read_h, addr);
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn lwu<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let addr = (rs1 + i.imm()) as u64;
    let v = mem!(p, read_w, addr);
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn lb<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let v = mem!(p, read_b, (rs1 + i.imm()) as u64);
    let sign_extended = ((v as i64) << 56 >> 56) as u64;
    p.regs.set(i.rd() as usize, sign_extended);
    p.advance_pc();
}

pub fn lh<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let v = mem!(p, read_h, (rs1 + i.imm()) as u64);
    let sign_extended = ((v as i64) << 48 >> 48) as u64;
    p.regs.set(i.rd() as usize, sign_extended);
    p.advance_pc();
}

pub fn lw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let v = mem!(p, read_w, (rs1 + i.imm()) as u64);
    let sign_extended = ((v as i64) << 32 >> 32) as u64;
    p.regs.set(i.rd() as usize, sign_extended);
    p.advance_pc();
}

pub fn ld<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let offset = (rs1 + i.imm()) as u64;
    let v = mem!(p, read_d, offset);
    p.regs.set(i.rd() as usize, v);
    p.advance_pc();
}

pub fn sb<M: Memory>(p: &mut Processor<M>, i: Stype) {
    let rs1 = p.regs.get(i.rs1() as usize) as i64;
    let rs2 = p.regs.get(i.rs2() as usize) as i64;
    let offset = rs1 + i.imm();
    mem!(p, write_b, offset as u64, rs2 as u8);
    p.advance_pc();
}

pub fn sh<M: Memory>(p: &mut Processor<M>, i: Stype) {
    let rs1 = p.regs.get(i.rs1() as usize) as i64;
    let rs2 = p.regs.get(i.rs2() as usize) as i64;
    let offset = rs1 + i.imm();
    mem!(p, write_h, offset as u64, rs2 as u16);
    p.advance_pc();
}

pub fn sw<M: Memory>(p: &mut Processor<M>, i: Stype) {
    let rs1 = p.regs.get(i.rs1() as usize) as i64;
    let rs2 = p.regs.get(i.rs2() as usize) as i64;
    let offset = rs1 + i.imm();
    mem!(p, write_w, offset as u64, rs2 as u32);
    p.advance_pc();
}

pub fn sd<M: Memory>(p: &mut Processor<M>, i: Stype) {
    let rs1 = p.regs.get(i.rs1() as usize) as i64;
    let rs2 = p.regs.get(i.rs2() as usize) as i64;
    let offset = rs1 + i.imm();
    mem!(p, write_d, offset as u64, rs2 as u64);
    p.advance_pc();
}
