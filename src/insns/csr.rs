use super::*;

pub trait Op {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, old: u64);
}

pub fn insn<M: Memory, O: Op>(p: &mut Processor<M>, i: Itype) {
    let old = handle_trap!(p, p.csrs.get(i.immu() as usize));
    O::exec(p, &i, old);
    p.regs.set(i.rd() as usize, old);
    p.advance_pc();
}

pub struct ReadWrite {}
impl Op for ReadWrite {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, _old: u64) {
        p.csrs.set(i.imm() as usize, p.regs.get(i.rs1() as usize))
    }
}

pub struct ReadWriteImm {}
impl Op for ReadWriteImm {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, _old: u64) {
        p.csrs.set(i.imm() as usize, i.rs1() as u64);
    }
}

pub struct ReadSet {}
impl Op for ReadSet {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, old: u64) {
        if i.rs1() != 0 {
            let rs1 = p.regs.get(i.rs1() as usize);
            p.csrs.set(i.immu() as usize, old | rs1);
        }
    }
}

pub struct ReadClear {}
impl Op for ReadClear {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, old: u64) {
        p.csrs
            .set(i.imm() as usize, old & !p.regs.get(i.rs1() as usize));
    }
}
