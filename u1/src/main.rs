#[inline(always)]
fn x(i: u64, low: u8, len: u8) -> u64 {
    i >> low & (1 << len) - 1
}

#[inline(always)]
fn rd(i: u32) -> u64 {
    x(i as u64, 7, 5)
}

#[inline(always)]
fn rs1(i: u32) -> u64 {
    x(i as u64, 15, 5)
}

#[inline(always)]
fn rs2(i: u32) -> u64 {
    x(i as u64, 20, 5)
}

struct I(u32);

trait TraitI {
    fn rd(&self) -> u64;
    fn rs1(&self) -> u64;
    fn imm(&self) -> i64;
}

impl TraitI for I {
    #[inline(always)] fn rd(&self) -> u64 { rd(self.0) }
    #[inline(always)] fn rs1(&self) -> u64 { rs1(self.0) }
    #[inline(always)] fn imm(&self) -> i64 { self.0 as i64 >> 20 }
}

impl Into<I> for u32 {
    fn into(self) -> I {
        I(self)
    }
}

struct R(u32);

trait TraitR {
    fn rd(&self) -> u64;
    fn rs1(&self) -> u64;
    fn rs2(&self) -> u64;
}

impl TraitR for R {
    #[inline(always)] fn rd(&self) -> u64 { rd(self.0) }
    #[inline(always)] fn rs1(&self) -> u64 { rs1(self.0) }
    #[inline(always)] fn rs2(&self) -> u64 { rs2(self.0) }
}

impl Into<R> for u32 {
    fn into(self) -> R {
        R(self)
    }
}

use std::env;

#[inline(never)]
fn sgibbsa(i: I) -> u32 {
    (i.rd() + i.rs1()) as u32
}

#[inline(never)]
fn sgibbsb(i: R) -> u32 {
    (i.rd() + i.rs2()) as u32
}

#[inline(never)]
fn work(i: u32) -> u32 {
    sgibbsa(i.into()) + sgibbsb(i.into())
}

pub fn main() {
    let n = env::current_exe().unwrap();
    let n = n.to_str().unwrap();
    let i = n.len() as u32;

    println!("{}", work(i));
}
