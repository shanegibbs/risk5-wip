use super::*;

// Load and Store

pub fn lbu<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let v = p.mem().read_b((rs1 + i.imm()) as u64);
    p.regs.set(i.rd() as usize, v as u64);
    p.advance_pc();
}

pub fn lw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let v = p.mem().read_w((rs1 + i.imm()) as u64);
    let sign_extended = ((v as i64) << 32 >> 32) as u64;
    p.regs.set(i.rd() as usize, sign_extended);
    p.advance_pc();
}

pub fn ld<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let offset = (rs1 + i.imm()) as u64;
    let v = p.mem().read_d(offset);
    p.regs.set(i.rd() as usize, v);
    p.advance_pc();
}

pub fn sb<M: Memory>(p: &mut Processor<M>, i: Stype) {
    let rs1 = p.regs.get(i.rs1() as usize) as i64;
    let rs2 = p.regs.get(i.rs2() as usize) as i64;
    let offset = rs1 + i.imm();
    p.mem_mut().write_b(offset as u64, rs2 as u8);
    p.advance_pc();
}

pub fn sh<M: Memory>(p: &mut Processor<M>, i: Stype) {
    let rs1 = p.regs.get(i.rs1() as usize) as i64;
    let rs2 = p.regs.get(i.rs2() as usize) as i64;
    let offset = rs1 + i.imm();
    p.mem_mut().write_h(offset as u64, rs2 as u16);
    p.advance_pc();
}

pub fn sw<M: Memory>(p: &mut Processor<M>, i: Stype) {
    let rs1 = p.regs.get(i.rs1() as usize) as i64;
    let rs2 = p.regs.get(i.rs2() as usize) as i64;
    let offset = rs1 + i.imm();
    p.mem_mut().write_w(offset as u64, rs2 as u32);
    p.advance_pc();
}

pub fn sd<M: Memory>(p: &mut Processor<M>, i: Stype) {
    let rs1 = p.regs.get(i.rs1() as usize) as i64;
    let rs2 = p.regs.get(i.rs2() as usize) as i64;
    let offset = rs1 + i.imm();
    p.mem_mut().write_d(offset as u64, rs2 as u64);
    p.advance_pc();
}
