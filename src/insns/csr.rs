use super::*;

pub trait Op {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, old: u64);
}

pub fn insn<M: Memory, O: Op>(p: &mut Processor<M>, i: Itype) {
    let old = match p.csrs().get(i.immu() as usize) {
        Ok(v) => v,
        Err(t) => {
            do_trap(p, t.cause, t.value);
            return;
        }
    };
    O::exec(p, &i, old);
    p.regs.set(i.rd() as usize, old);
    p.advance_pc();
}

pub struct ReadWrite {}
impl Op for ReadWrite {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, _old: u64) {
        p.set_csr(i.imm() as u32, p.get_reg(i.rs1()))
    }
}

pub struct ReadWriteImm {}
impl Op for ReadWriteImm {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, _old: u64) {
        p.set_csr(i.imm() as u32, i.rs1() as u64)
    }
}

pub struct ReadClearImm {}
impl Op for ReadClearImm {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, old: u64) {
        p.set_csr(i.imm() as u32, old & !(i.rs1() as u64))
    }
}

pub struct ReadSet {}
impl Op for ReadSet {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, old: u64) {
        if i.rs1() != 0 {
            let rs1 = p.regs.get(i.rs1() as usize);
            p.set_csr(i.immu() as u32, old | rs1)
        }
    }
}

pub struct ReadClear {}
impl Op for ReadClear {
    fn exec<M>(p: &mut Processor<M>, i: &Itype, old: u64) {
        p.set_csr(i.imm() as u32, old & !p.get_reg(i.rs1() as u32))
    }
}
