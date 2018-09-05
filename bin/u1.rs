extern crate risk5;

use risk5::Instruction;
use std::env;

#[inline(never)]
fn doit(n: u32) -> i64 {
    // let n = 0x84dff0ef;
    n.jimm20()
}

fn main() {
    let i: u32 = env::args().next().unwrap().parse().unwrap();
    println!("{}", doit(i));
}
