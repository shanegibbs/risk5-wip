#![feature(use_extern_macros)]

extern crate pretty_env_logger;
extern crate derive_insn;
extern crate risk5;

// https://github.com/termoshtt/deco/blob/master/src/lib.rs

use risk5::Processor;
use std::env;

// struct Insns;

mod insns {

    use derive_insn::insn;
    use risk5::Processor;

    #[insn(kind=I,mask=110,match=100)]
    pub fn addi(p: &mut Processor, rd: u32, rs: u32, imm: u32) {
        let v = p.regs.get(rs) + imm as u64;
        p.regs.set(rd as usize, v);
        p.advance_pc();
    }

}

// #[inline(never)]
// fn doit(p: &mut Processor, n: u32) {}

fn main() {
	pretty_env_logger::init();

    let mut args = env::args();
    args.next();
    let i: u32 = args.next().unwrap().parse().unwrap();
    println!("i={}", i);

    let mut p = Processor::new(0);
    // doit(&mut p, i);
    // println!("{}", p.regs.get(0 as u64));

    // print_abc(3);
    use insns::*;
    addi(&mut p, 1, 2, 10);
    addi_exec(&mut p, i);
    println!("{}", addi_desc());
}
