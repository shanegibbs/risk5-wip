pub trait Instruction {
    fn rd(&self) -> usize;
    fn u_imm(&self) -> i64;
    fn i_imm(&self) -> i64;

    fn rs1(&self) -> u32 {
        unimplemented!()
    }
    fn rs2(&self) -> u32 {
        unimplemented!()
    }

    fn imm12(&self) -> u32 {
        unimplemented!()
    }
    fn imm12lo(&self) -> u32 {
        unimplemented!()
    }
    fn imm12hi(&self) -> u32 {
        unimplemented!()
    }

    fn imm20(&self) -> u32 {
        unimplemented!()
    }

    fn bimm12lo(&self) -> u32 {
        unimplemented!()
    }
    fn bimm12hi(&self) -> u32 {
        unimplemented!()
    }

    fn jimm20(&self) -> i64 {
        unimplemented!()
    }

    fn aqrl(&self) -> u32 {
        unimplemented!()
    }
    fn succ(&self) -> u32 {
        unimplemented!()
    }
    fn fm(&self) -> u32 {
        unimplemented!()
    }
    fn pred(&self) -> u32 {
        unimplemented!()
    }
    fn shamt(&self) -> u32 {
        unimplemented!()
    }
    fn shamtw(&self) -> u32 {
        unimplemented!()
    }
}

#[inline(always)]
fn x(i: u64, low: u8, len: u8) -> u64 {
    i >> low & (1 << len) - 1
}

// extend bits up to the sign bit and then back down again
#[inline(always)]
fn sign_extend<T: Into<i64>>(i: T, len: u8) -> i64 {
    let extend = 64 - len;
    (i.into()) << extend >> extend
}

impl Instruction for u32 {
    #[inline(always)]
    fn rd(&self) -> usize {
        x(*self as u64, 7, 5) as usize
    }

    #[inline(always)]
    fn rs1(&self) -> u32 {
        x(*self as u64, 15, 5) as u32
    }
    #[inline(always)]
    fn rs2(&self) -> u32 {
        x(*self as u64, 20, 5) as u32
    }

    #[inline(always)]
    fn u_imm(&self) -> i64 {
        return *self as i64 >> 12 << 12;
    }

    #[inline(always)]
    fn i_imm(&self) -> i64 {
        return *self as i64 >> 20;
    }

    #[inline(always)]
    fn imm12(&self) -> u32 {
        *self >> 20
    }
    #[inline(always)]
    fn imm12lo(&self) -> u32 {
        // (x(8, 4) << 1) + (x(25, 6) << 5) + (x(7, 1) << 11) + (imm_sign() << 12)
        0
    }
    #[inline(always)]
    fn imm12hi(&self) -> u32 {
        0
    }

    #[inline(always)]
    fn imm20(&self) -> u32 {
        return (*self as i64 >> 20) as u32;
    }

    #[inline(always)]
    fn bimm12lo(&self) -> u32 {
        0
    }
    #[inline(always)]
    fn bimm12hi(&self) -> u32 {
        0
    }

    #[inline(always)]
    fn jimm20(&self) -> i64 {
        let s = *self as u64;
        let mut unsigned = 0;
        unsigned |= x(s, 21, 10) << 1;
        unsigned |= x(s, 20, 1) << 11;
        unsigned |= x(s, 12, 8) << 12;
        unsigned |= x(s, 31, 1) << 20;
        sign_extend(unsigned as i64, 20)
    }

    // shift amount
    #[inline(always)]
    fn shamt(&self) -> u32 {
        self.imm20() & 0x1f
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_env_logger;

    #[test]
    fn rd() {
        let _ = pretty_env_logger::try_init();
        assert_eq!(0x2000006.rd(), 0);
        assert_eq!(0x05013503.rd(), 10);
        assert_eq!(0x01813183.rd(), 3);
    }

    #[test]
    fn jimm20() {
        let _ = pretty_env_logger::try_init();
        // debug!("starting");

        //80002d60:   84dff0ef                jal     ra,800025ac <poweroff>
        let n = 0x84dff0ef;
        let offset = 0x800025ac - 0x80002d60;
        // debug!("insn   {i:x}, {i:b}", i = n);
        // debug!("offset {i:x}, {i:b}", i = offset as i32);
        // debug!("jimm20 {i:x}, {i:b}", i = n.jimm20() as i32);
        assert_eq!(n.jimm20(), offset);

        fn t(pc: u32, insn: u32, target: u32) {
            let offset = target as i32 - pc as i32;
            let jimm20 = insn.jimm20();
            // debug!("insn   {i:x}, {i:b}", i = insn);
            // debug!("offset {i:x}, {i:b}", i = offset);
            // debug!("jimm20 {i:x}, {i:b}", i = jimm20 as i32);
            assert_eq!(jimm20, offset as i64);
        }

        t(0x80002bbc, 0xc2dff0ef, 0x800027e8);
        t(0x80002bc0, 0xc7dff0ef, 0x8000283c);
        t(0x80002bc4, 0xd21ff0ef, 0x800028e4);
        t(0x80002bc8, 0xdbdff0ef, 0x80002984);
        t(0x80002bdc, 0xf14fd06f, 0x800002f0);
        t(0x80002d54, 0xfc0ff0ef, 0x80002514);
        t(0x80002d60, 0x84dff0ef, 0x800025ac);
        t(0x80002d80, 0xf94ff0ef, 0x80002514);
        t(0x80002d8c, 0x821ff0ef, 0x800025ac);
        t(0x80002db4, 0x314050ef, 0x800080c8);
    }
}
