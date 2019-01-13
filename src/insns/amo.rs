use super::*;

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

pub fn amoswapw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    let rs1 = p.regs.geti(i.rs1() as usize);
    let v = mem!(p, read_w, rs1 as u64);

    let sign_extended_v = ((v as i64) << 32 >> 32) as u64;

    let rs2 = p.regs.get(i.rs2() as usize);

    mem!(p, write_w, rs1 as u64, rs2 as u32);

    p.regs.set(i.rd() as usize, sign_extended_v);

    p.advance_pc();
}

pub fn amoaddw<M: Memory>(p: &mut Processor<M>, i: Rtype) {
    // load value from addr(rs1)
    let addr = p.regs.get(i.rs1() as usize);
    let v = mem!(p, read_w, addr);

    // sign extend and place in rd
    let sign_extended_v = ((v as i64) << 32 >> 32) as u64;
    // other value
    let rs2 = p.regs.geti(i.rs2() as usize);

    // do operation
    let result = sign_extended_v as i64 + rs2;

    // write result back to addr
    mem!(p, write_w, addr, result as u32);

    p.regs.set(i.rd() as usize, sign_extended_v);

    p.advance_pc();
}
