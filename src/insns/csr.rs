use super::*;

pub fn csrrw<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let old = handle_trap!(p, p.csrs.get(i.imm() as usize));
    p.csrs.set(i.imm() as usize, p.regs.get(i.rs1() as usize));
    p.regs.set(i.rd() as usize, old);
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

pub fn csrrc<M: Memory>(p: &mut Processor<M>, i: Itype) {
    let old = handle_trap!(p, p.csrs.get(i.imm() as usize));
    p.csrs
        .set(i.imm() as usize, old & !p.regs.get(i.rs1() as usize));
    p.regs.set(i.rd() as usize, old);
    p.advance_pc();
}
