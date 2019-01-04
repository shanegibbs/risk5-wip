use crate::itypes::*;
use crate::*;

pub fn amoswapw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let v = p.mem().read_w(rs1 as u64);
    let sign_extended = ((v as i64) << 32 >> 32) as u64;
    p.regs.set(i.rd() as usize, sign_extended);
    p.advance_pc();
}
