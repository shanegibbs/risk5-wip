use crate::Processor;
use std::fmt;

pub type Regi = u32;

#[inline(always)]
fn sign_extend(val: u64, len: u8) -> i64 {
    let extend = 64 - len;
    (val as i64) << extend >> extend
}

pub trait Base {
    #[inline(always)]
    fn val(&self) -> u32;

    #[inline(always)]
    fn field(&self, offset: u8, len: u8) -> Regi {
        self.val() >> offset & (1 << len) - 1
    }

    // extend bits up to the sign bit and then back down again
    fn signed_field(&self, offset: u8, len: u8) -> i64 {
        sign_extend(self.field(offset, len) as u64, len)
    }
}

pub trait FieldRd: Base {
    #[inline(always)]
    fn rd(&self) -> Regi {
        self.field(7, 5)
    }
}

pub trait FieldRs1: Base {
    #[inline(always)]
    fn rs1(&self) -> Regi {
        self.field(15, 5)
    }
}

pub trait FieldRs2: Base {
    #[inline(always)]
    fn rs2(&self) -> Regi {
        self.field(20, 5)
    }
}

// B Instruction Type

pub struct Btype(u32);

impl Base for Btype {
    #[inline(always)]
    fn val(&self) -> u32 {
        self.0
    }
}

impl Btype {
    #[inline(always)]
    pub fn imm(&self) -> i64 {
        let mut i: u32 = 0;
        i |= self.field(8, 4) << 1;
        i |= self.field(25, 6) << 5;
        i |= self.field(7, 1) << 11;
        i |= self.field(31, 1) << 12;
        sign_extend(i as u64, 12)
    }

    #[inline(always)]
    pub fn jump<M>(&self, p: &mut Processor<M>) {
        let new_pc = p.pc() as i64 + self.imm();
        p.set_pc(new_pc as u64);
    }
}

impl FieldRs1 for Btype {}
impl FieldRs2 for Btype {}

impl Into<Btype> for u32 {
    fn into(self) -> Btype {
        Btype(self)
    }
}

impl fmt::Display for Btype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} 0x{:x}", self.rs1(), self.rs2(), self.imm())
    }
}

// R Instruction Type

pub struct Rtype(u32);

impl Base for Rtype {
    #[inline(always)]
    fn val(&self) -> u32 {
        self.0
    }
}

impl FieldRd for Rtype {}
impl FieldRs1 for Rtype {}
impl FieldRs2 for Rtype {}

impl Into<Rtype> for u32 {
    fn into(self) -> Rtype {
        Rtype(self)
    }
}

impl fmt::Display for Rtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.rd(), self.rs1(), self.rs2())
    }
}

// I Instruction Type

pub struct Itype(u32);

impl Base for Itype {
    #[inline(always)]
    fn val(&self) -> u32 {
        self.0
    }
}

impl FieldRd for Itype {}
impl FieldRs1 for Itype {}

impl Itype {
    #[inline(always)]
    pub fn immu(&self) -> u64 {
        self.field(20, 12) as u64
    }

    #[inline(always)]
    pub fn imm(&self) -> i64 {
        self.signed_field(20, 12)
    }
}

impl Into<Itype> for u32 {
    fn into(self) -> Itype {
        Itype(self)
    }
}

impl fmt::Display for Itype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} 0x{:x}", self.rd(), self.rs1(), self.imm())
    }
}

// S Instruction Type

pub struct Stype(u32);

impl Base for Stype {
    #[inline(always)]
    fn val(&self) -> u32 {
        self.0
    }
}

impl FieldRs1 for Stype {}
impl FieldRs2 for Stype {}

impl Stype {
    #[inline(always)]
    pub fn imm(&self) -> i64 {
        let mut i: u32 = 0;
        i |= self.field(7, 5);
        i |= self.field(25, 7) << 5;
        sign_extend(i as u64, 12)
    }
}

impl Into<Stype> for u32 {
    fn into(self) -> Stype {
        Stype(self)
    }
}

impl fmt::Display for Stype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} 0x{:x}", self.rs1(), self.rs2(), self.imm())
    }
}

// U Instruction Type

pub struct Utype(u32);

impl Base for Utype {
    #[inline(always)]
    fn val(&self) -> u32 {
        self.0
    }
}

impl FieldRd for Utype {}

impl Utype {
    #[inline(always)]
    pub fn imm(&self) -> i64 {
        let i = (self.val() as i32) >> 12 << 12;
        i as i64
    }
}

impl Into<Utype> for u32 {
    fn into(self) -> Utype {
        Utype(self)
    }
}

impl fmt::Display for Utype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.imm())
    }
}

// J Instruction Type

pub struct Jtype(u32);

impl Base for Jtype {
    #[inline(always)]
    fn val(&self) -> u32 {
        self.0
    }
}

impl FieldRd for Jtype {}

impl Jtype {
    #[inline(always)]
    pub fn imm(&self) -> i64 {
        let mut i: u32 = 0;
        i |= self.field(21, 10) << 1;
        i |= self.field(20, 1) << 11;
        i |= self.field(12, 8) << 12;
        i |= self.field(31, 1) << 20;
        sign_extend(i as u64, 20)
    }
}

impl Into<Jtype> for u32 {
    fn into(self) -> Jtype {
        Jtype(self)
    }
}

impl fmt::Display for Jtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.imm())
    }
}
