#[inline(always)]
fn x(i: u64, low: u8, len: u8) -> u64 {
    i >> low & (1 << len) - 1
}

struct I(u32);

trait TraitI {
    fn rd(&self) -> u64;
    fn rs1(&self) -> u64;
    fn imm(&self) -> i64;
}

impl TraitI for I {
    #[inline(always)]
    fn rd(&self) -> u64 {
        x(self.0 as u64, 7, 5)
    }
    #[inline(always)]
    fn rs1(&self) -> u64 {
        x(self.0 as u64, 15, 5)
    }
    #[inline(always)]
    fn imm(&self) -> i64 {
        return self.0 as i64 >> 20;
    }
}

struct R(u32);

trait TraitR {
    fn rd(&self) -> u64;
    fn rs1(&self) -> u64;
    fn rs2(&self) -> u64;
}

impl TraitR for R {
    #[inline(always)]
    fn rd(&self) -> u64 {
        x(self.0 as u64, 7, 5)
    }
    #[inline(always)]
    fn rs1(&self) -> u64 {
        x(self.0 as u64, 15, 5)
    }
    #[inline(always)]
    fn rs2(&self) -> u64 {
        x(self.0 as u64, 20, 5)
    }
}

pub fn main() {
    let i: I = I(0);
    println!("{} {}", i.rd(), i.rs1());
    println!("{} {}", i.rd(), i.rs1());
}
