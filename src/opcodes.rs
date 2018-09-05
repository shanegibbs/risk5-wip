use mmu::Memory;
use csrs::Csrs;
use insn::Instruction;

static _REG_NAMES: &'static [&str] = &["zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0/fp",
                                       "s1"];

struct Regs {
    regs: [u64; 32],
}

impl Regs {
    fn new() -> Self {
        Regs { regs: [0; 32] }
    }
    fn get<T: Into<u64>>(&self, i: T) -> u64 {
        let i = i.into();
        // reg 0 is a black hole
        if i == 0 {
            return 0;
        }
        debug!("Getting reg 0x{:x}", i);
        self.regs[i as usize]
    }
    fn set(&mut self, i: usize, v: u64) {
        // reg 0 is a black hole
        if i == 0 {
            return;
        }
        debug!("Setting reg 0x{:x} to 0x{:x}", i, v);
        self.regs[i] = v;
    }
}

pub struct Processor {
    pc: u64,
    regs: Regs,
}

impl Processor {
    pub fn new(pc: u64) -> Self {
        Processor {
            pc: pc,
            regs: Regs::new(),
        }
    }

    pub fn step(&mut self, csrs: &mut Csrs, mem: &mut Memory) {
        let insn = mem.read_w(self.pc);
        trace!("0x{:x} inst 0x{:x}", self.pc, insn);
        self.handle_inst(insn, csrs, mem)
    }

    fn advance_pc(&mut self) {
        self.pc += 4;
    }

    fn set_pc(&mut self, pc: u64) {
        debug!("0x{:x} > Setting pc to 0x{:x}", self.pc, pc);
        self.pc = pc;
    }

    fn handle_inst(&mut self, insn: u32, csrs: &mut Csrs, mut m: &mut Memory) {
        macro_rules! unmatched_insn {() => (panic!(format!("No inst match 0x{:x}", insn)))}
        macro_rules! unimplemented_insn {($name:expr) => ( panic!("No impl for {}", $name))}

        // macro_rules! ld_inst {() => (self.regs[i.rd()] = m.read_d(i.rs1() as u64 + i.i_imm() as u64))}
        macro_rules! auipc_insn {($id:expr,$imm:expr) => (self.regs.set($id, self.pc + $imm as u64); self.advance_pc())}
        macro_rules! addi_insn {($rd:expr,$rs1:expr,$imm:expr) => (let v = self.regs.get($rs1) + $imm as u64; self.regs.set($rd, v); self.advance_pc())}
        macro_rules! jal_insn {($rd:expr,$imm:expr) => (let new_pc = (self.pc as i64 + $imm) as u64; self.set_pc(new_pc))}

        macro_rules! bne_insn {($immhi:expr,$rs1:expr,$rs2:expr,$immlo:expr) => (if self.regs.get($rs1) != self.regs.get($rs2) {let new_pc = self.pc + ($immhi | $immlo) as u64; self.set_pc(new_pc)} else {self.advance_pc()} )}

        macro_rules! csrrw_insn {($rd:expr,$rs1:expr,$csr:expr) => (self.regs.set($rd, csrs.get($csr)); csrs.set($csr, self.regs.get($rs1)); self.advance_pc();)}
        macro_rules! csrrs_insn {($rd:expr,$rs1:expr,$csr:expr) => (self.regs.set($rd, csrs.get($csr) | $rs1 as u64); self.advance_pc();)}

        include!(concat!(env!("OUT_DIR"), "/insns.rs"));
    }
}

fn addi(p: &mut Processor, insn: u32) {
    let v = p.regs.get(insn.rs1()) + insn.imm20() as u64;
    p.regs.set(insn.rd(), v);
    p.advance_pc();
}

fn get_opcode_fn(insn: u32) -> fn(&mut Processor, u32) -> () {
    addi
}
