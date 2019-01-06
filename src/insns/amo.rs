use super::*;

pub fn amoswapw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let v = p.mem().read_w(rs1 as u64);
    error!("v=0x{:x}", v);

    let sign_extended_v = ((v as i64) << 32 >> 32) as u64;
    error!("v=0x{:x}", sign_extended_v);

    let rs2 = p.regs.get(i.rs2() as usize);

    p.mem_mut().write_w(rs1 as u64, rs2 as u32);

    p.regs.set(i.rd() as usize, sign_extended_v);

    error!("rs1=0x{:x}", rs1);
    error!("rs2=0x{:x}", rs2);

    p.advance_pc();
}

pub fn amoaddw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    // load value from addr(rs1)
    let addr = p.regs.get(i.rs1() as usize);
    let v = p.mem().read_w(addr);

    // sign extend and place in rd
    let sign_extended_v = ((v as i64) << 32 >> 32) as u64;
    p.regs.set(i.rd() as usize, sign_extended_v);

    // other value
    let rs2 = p.regs.geti(i.rs2() as usize);

    // do operation
    let result = sign_extended_v as i64 + rs2;

    // write result back to addr
    p.mem_mut().write_w(addr, result as u32);

    p.advance_pc();
}
