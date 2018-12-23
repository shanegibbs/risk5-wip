pub type Regi = u8;

#[inline(always)]
fn x(i: u64, low: u8, len: u8) -> u64 {
    i >> low & (1 << len) - 1
}

// extend bits up to the sign bit and then back down again
#[inline(always)]
pub fn sign_extend<T: Into<u64>>(i: T, len: u8) -> i64 {
    let extend = 64 - len;
    (Into::<u64>::into(i) as i64) << extend >> extend
}

#[inline(always)]
fn rd(i: u32) -> Regi {
    x(i as u64, 7, 5) as Regi
}

#[inline(always)]
fn rs1(i: u32) -> Regi {
    x(i as u64, 15, 5) as Regi
}

#[inline(always)]
fn rs2(i: u32) -> Regi {
    x(i as u64, 20, 5) as Regi
}

pub struct Itype(u32);

pub trait TraitI {
    fn rd(&self) -> Regi;
    fn rs1(&self) -> Regi;
    fn imm(&self) -> i64;
}

impl TraitI for Itype {
    #[inline(always)] fn rd(&self) -> Regi { rd(self.0) }
    #[inline(always)] fn rs1(&self) -> Regi { rs1(self.0) }
    #[inline(always)] fn imm(&self) -> i64 { sign_extend(self.0 >> 20, 12) }
}

impl Into<Itype> for u32 {
    fn into(self) -> Itype {
        Itype(self)
    }
}

struct Rtype(u32);

trait TraitR {
    fn rd(&self) -> Regi;
    fn rs1(&self) -> Regi;
    fn rs2(&self) -> Regi;
}

impl TraitR for Rtype {
    #[inline(always)] fn rd(&self) -> Regi { rd(self.0) }
    #[inline(always)] fn rs1(&self) -> Regi { rs1(self.0) }
    #[inline(always)] fn rs2(&self) -> Regi { rs2(self.0) }
}

impl Into<Rtype> for u32 {
    fn into(self) -> Rtype {
        Rtype(self)
    }
}
