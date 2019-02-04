use crate::itypes::*;
use crate::Processor;

pub fn reg<M, O: Operation<i64>>(p: &mut Processor<M>, i: Rtype) {
    computation::<M, Rtype, i64, O, Rs1Arg, Rs2Arg>(p, i);
}

pub fn regw<M, O: Operation<i32>>(p: &mut Processor<M>, i: Rtype) {
    computation::<M, Rtype, i32, O, Rs1Arg, Rs2Arg>(p, i);
}

pub fn imm<M, O: Operation<i64>>(p: &mut Processor<M>, i: Itype) {
    computation::<M, Itype, i64, O, Rs1Arg, ImmArg>(p, i);
}

pub fn immw<M, O: Operation<i32>>(p: &mut Processor<M>, i: Itype) {
    computation::<M, Itype, i32, O, Rs1Arg, ImmArg>(p, i);
}

pub trait Operation<T> {
    fn exec(lhs: T, rhs: T) -> u64;
}

trait Arg<M, I, T> {
    fn arg(p: &mut Processor<M>, i: &I) -> T;
}

fn computation<M, I, T, O, LA, RA>(p: &mut Processor<M>, i: I)
where
    I: FieldRd,
    O: Operation<T>,
    LA: Arg<M, I, T>,
    RA: Arg<M, I, T>,
{
    let lhs = LA::arg(p, &i);
    let rhs = RA::arg(p, &i);
    let result = O::exec(lhs, rhs);
    p.regs.set(i.rd() as usize, result);
    p.advance_pc();
}

struct Rs1Arg;
impl<M, I: FieldRs1> Arg<M, I, i64> for Rs1Arg {
    fn arg(p: &mut Processor<M>, i: &I) -> i64 {
        p.regs.geti(i.rs1() as usize)
    }
}
impl<M, I: FieldRs1> Arg<M, I, i32> for Rs1Arg {
    fn arg(p: &mut Processor<M>, i: &I) -> i32 {
        p.regs.geti(i.rs1() as usize) as i32
    }
}

struct Rs2Arg;
impl<M, I: FieldRs2> Arg<M, I, i64> for Rs2Arg {
    fn arg(p: &mut Processor<M>, i: &I) -> i64 {
        p.regs.geti(i.rs2() as usize)
    }
}
impl<M, I: FieldRs2> Arg<M, I, i32> for Rs2Arg {
    fn arg(p: &mut Processor<M>, i: &I) -> i32 {
        p.regs.geti(i.rs2() as usize) as i32
    }
}

struct ImmArg;
impl<M, I: FieldImm> Arg<M, I, i64> for ImmArg {
    fn arg(_: &mut Processor<M>, i: &I) -> i64 {
        i.imm()
    }
}
impl<M, I: FieldImm> Arg<M, I, i32> for ImmArg {
    fn arg(_: &mut Processor<M>, i: &I) -> i32 {
        i.imm() as i32
    }
}

fn sign_extend(i: i32) -> u64 {
    ((i as i64) << 32 >> 32) as u64
}

pub struct Add;
impl Operation<i64> for Add {
    fn exec(lhs: i64, rhs: i64) -> u64 {
        (lhs + rhs) as u64
    }
}
impl Operation<i32> for Add {
    fn exec(lhs: i32, rhs: i32) -> u64 {
        sign_extend(lhs + rhs)
    }
}

pub struct And;
impl Operation<i64> for And {
    fn exec(lhs: i64, rhs: i64) -> u64 {
        (lhs & rhs) as u64
    }
}
