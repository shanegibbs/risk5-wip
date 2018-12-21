use mmu::*;
use csrs::Csrs;
use std::fmt;

pub struct Matcher<M: Memory> {
    mask: u32,
    mtch: u32,
    exec: fn(&mut Processor<M>, u32),
}

impl<M: Memory> Matcher<M> {
    pub fn new(mask: u32, mtch: u32, exec: fn(&mut Processor<M>, u32)) -> Self {
        Self { mask, mtch, exec }
    }
}

impl<M: fmt::Debug + Memory> fmt::Debug for Matcher<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matcher")
    }
}

pub static REG_NAMES: &'static [&str] = &["zero", "ra", "sp", "gp", "tp",
                                        "t0", "t1", "t2",
                                        "s0", "s1",
                                        "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7",
                                        "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "sA", "sB",
                                        "t3", "t4", "t5", "t6"];

#[derive(Debug)]
pub struct Regs {
    regs: [u64; 32],
}

impl Regs {
    fn new() -> Self {
        Regs { regs: [0; 32] }
    }

    #[inline(always)]
    pub fn get<T: Into<usize>>(&self, i: T) -> u64 {
        let i = i.into();
        let v = self.regs[i];
        // trace!("Getting reg 0x{:x} 0x{:x}", i, v);
        v
    }

    #[inline(always)]
    pub fn set(&mut self, i: usize, v: u64) {
        // reg 0 is a black hole
        if i == 0 {
            return;
        }
        trace!("Setting reg 0x{:x} 0x{:x}", i, v);
        self.regs[i] = v;
    }
}

#[derive(Debug)]
pub struct Processor<M> {
    pc: u64,
    pub regs: Regs,
    pub csrs: Csrs,
    mem: M,
}

impl Processor<FakeMemory> {
    pub fn get_mem(&mut self) -> &mut FakeMemory {
        &mut self.mem
    }
}

impl<M> Processor<M> {
    pub fn new(pc: u64, mem: M) -> Self {
        Processor {
            pc: pc,
            regs: Regs::new(),
            csrs: Csrs::new(),
            mem,
        }
    }

    pub fn mem(&mut self) -> &M {
        &self.mem
    }

    pub fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }

    pub fn step(&mut self, matchers: &[Matcher<M>])
        where M: Memory
    {
        let insn = self.mem.read_w(self.pc);
        trace!("0x{:x} inst 0x{:x}", self.pc, insn);
        for matcher in matchers {
            if insn & matcher.mask == matcher.mtch {
                (matcher.exec)(self, insn);
                return;
            }
        }
        panic!(format!("Unmatched instruction: 0x{:x}", insn));
    }

    #[inline(always)]
    pub fn advance_pc(&mut self) {
        self.pc += 4;
    }

    #[inline(always)]
    pub fn set_pc(&mut self, pc: u64) {
        trace!("0x{:x} > Setting pc to 0x{:x}", self.pc, pc);
        self.pc = pc;
    }

    #[inline(always)]
    pub fn pc(&self) -> u64 {
        self.pc
    }

    fn _handle_inst(&mut self, insn: u32, _csrs: &mut Csrs, mut _m: &mut Memory) {
        panic!(format!("Unmatched instruction: 0x{:x}", insn));

        /*
        macro_rules! unmatched_insn {() => (panic!(format!("No inst match 0x{:x}", insn)))}
        macro_rules! unimplemented_insn {($name:expr) => ( panic!("No impl for {}", $name))}

        // macro_rules! ld_inst {() => (self.regs[i.rd()] = m.read_d(i.rs1() as u64 + i.i_imm() as u64))}
        macro_rules! auipc_insn {($id:expr,$imm:expr) => (self.regs.set($id, self.pc + $imm as u64); self.advance_pc())}
        // macro_rules! addi_insn {($rd:expr,$rs1:expr,$imm:expr) => (let v = self.regs.get($rs1) + $imm as u64; self.regs.set($rd, v); self.advance_pc())}
        macro_rules! addi_insn {($rd:expr,$rs1:expr,$imm:expr) => (addi_1(self, $rd, $rs1, $imm))}
        macro_rules! jal_insn {($rd:expr,$imm:expr) => (let new_pc = (self.pc as i64 + $imm) as u64; self.set_pc(new_pc))}

        macro_rules! bne_insn {($immhi:expr,$rs1:expr,$rs2:expr,$immlo:expr) => (if self.regs.get($rs1) != self.regs.get($rs2) {let new_pc = self.pc + ($immhi | $immlo) as u64; self.set_pc(new_pc)} else {self.advance_pc()} )}

        macro_rules! csrrw_insn {($rd:expr,$rs1:expr,$csr:expr) => (self.regs.set($rd, csrs.get($csr)); csrs.set($csr, self.regs.get($rs1)); self.advance_pc();)}
        macro_rules! csrrs_insn {($rd:expr,$rs1:expr,$csr:expr) => (self.regs.set($rd, csrs.get($csr) | $rs1 as u64); self.advance_pc();)}

        include!(concat!(env!("OUT_DIR"), "/insns.rs"));
        */
    }
}
