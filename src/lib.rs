#[macro_use]
extern crate log;
extern crate elf;
extern crate pretty_env_logger;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod csrs;
mod elf_loader;
mod insns;
mod itypes;
pub mod log_runner;
mod mmu;
mod mstatus;
mod opcodes;
use std::fs::File;
use std::io::Read;

pub use insns::*;
use mmu::*;
pub use opcodes::*;

pub fn risk5_main() {
    pretty_env_logger::init();

    let mut mem = BlockMemory::new(15);

    use std::env;
    let filename = env::var("BIN").unwrap_or("assets/bbl".into());

    let (entry, sections) = elf_loader::read_program_segments(&filename);
    let mut elf = File::open(&filename).unwrap();
    let mut file_bytes = vec![];
    let _read_file_size = elf.read_to_end(&mut file_bytes).unwrap();

    debug!("Loading ELF");
    for (f_offset, m_offset, size) in sections {
        mem.add_block(m_offset, size);
        for i in 0..size {
            mem.write_b(m_offset + i, file_bytes[(f_offset + i) as usize]);
        }
    }

    //  auipc   t0, 0x0
    //  addi    a1, t0, 32
    //  csrr    a0, mhartid
    //  ld      t0, 24(t0)
    //  jr      t0
    //  j       pc + 0x200
    //  li      ra, 0
    // let mem = vec![
    //     0x00000297, 0x02028593, 0xf1402573, 0x0182b283, 0x00028067, 0x2000006f, 0x00000093,v
    // ];

    // mem.add_block(0, 10);
    // // 00e00513
    // mem.write_b(0, 0x00);
    // mem.write_b(1, 0xe0);v
    // mem.write_b(2, 0x05);v
    // mem.write_b(3, 0x13);

    let matchers = build_matchers();

    // let mut csrs = csrs::Csrs::new();
    let mut cpu = opcodes::Processor::new(entry, mem);
    loop {
        cpu.step(&matchers);
    }
}

fn build_matchers<M: Memory>() -> Vec<Matcher<M>> {
    macro_rules! wrap {
        ($f:ident) => {
            |p, i| {
                let i = i.into();
                debug!("{} {}", stringify!($f), i);
                $f(p, i)
            }
        };
    }

    vec![
        Matcher::new(0x707f, 0x63, wrap!(beq)),
        Matcher::new(0x707f, 0x1063, wrap!(bne)),
        Matcher::new(0x707f, 0x4063, wrap!(blt)),
        Matcher::new(0x707f, 0x5063, wrap!(bge)),
        Matcher::new(0x707f, 0x6063, wrap!(bltu)),
        Matcher::new(0x707f, 0x7063, wrap!(bgeu)),
        Matcher::new(0x707f, 0x67, wrap!(jalr)),
        Matcher::new(0x7f, 0x6f, wrap!(jal)),
        Matcher::new(0x7f, 0x37, wrap!(lui)),
        Matcher::new(0x7f, 0x17, wrap!(auipc)),
        Matcher::new(0x707f, 0x13, wrap!(addi)),
        Matcher::new(0xfc00707f, 0x1013, wrap!(slli)),
        Matcher::new(0x707f, 0x2013, |p, _| {
            panic!(format!("Unimplemented insn 'slti' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x3013, |p, _| {
            panic!(format!("Unimplemented insn 'sltiu' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x4013, wrap!(xori)),
        Matcher::new(0xfc00707f, 0x5013, wrap!(srli)),
        Matcher::new(0xfc00707f, 0x40005013, wrap!(srai)),
        Matcher::new(0x707f, 0x6013, wrap!(ori)),
        Matcher::new(0x707f, 0x7013, wrap!(andi)),
        Matcher::new(0xfe00707f, 0x33, wrap!(add)),
        Matcher::new(0xfe00707f, 0x40000033, |p, _| {
            panic!(format!("Unimplemented insn 'sub' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x1033, |p, _| {
            panic!(format!("Unimplemented insn 'sll' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2033, |p, _| {
            panic!(format!("Unimplemented insn 'slt' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x3033, |p, _| {
            panic!(format!("Unimplemented insn 'sltu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x4033, wrap!(xor)),
        Matcher::new(0xfe00707f, 0x5033, |p, _| {
            panic!(format!("Unimplemented insn 'srl' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x40005033, |p, _| {
            panic!(format!("Unimplemented insn 'sra' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x6033, wrap!(or)),
        Matcher::new(0xfe00707f, 0x7033, wrap!(and)),
        Matcher::new(0x707f, 0x1b, wrap!(addiw)),
        Matcher::new(0xfe00707f, 0x101b, wrap!(slliw)),
        Matcher::new(0xfe00707f, 0x501b, wrap!(srliw)),
        Matcher::new(0xfe00707f, 0x4000501b, wrap!(sraiw)),
        Matcher::new(0xfe00707f, 0x3b, |p, _| {
            panic!(format!("Unimplemented insn 'addw' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x4000003b, wrap!(subw)),
        Matcher::new(0xfe00707f, 0x103b, |p, _| {
            panic!(format!("Unimplemented insn 'sllw' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x503b, |p, _| {
            panic!(format!("Unimplemented insn 'srlw' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x4000503b, |p, _| {
            panic!(format!("Unimplemented insn 'sraw' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x3, |p, _| {
            panic!(format!("Unimplemented insn 'lb' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x1003, |p, _| {
            panic!(format!("Unimplemented insn 'lh' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x2003, wrap!(lw)),
        Matcher::new(0x707f, 0x3003, wrap!(ld)),
        Matcher::new(0x707f, 0x4003, wrap!(lbu)),
        Matcher::new(0x707f, 0x5003, |p, _| {
            panic!(format!("Unimplemented insn 'lhu' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x6003, |p, _| {
            panic!(format!("Unimplemented insn 'lwu' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x23, |p, _| {
            panic!(format!("Unimplemented insn 'sb' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x1023, |p, _| {
            panic!(format!("Unimplemented insn 'sh' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x2023, wrap!(sw)),
        Matcher::new(0x707f, 0x3023, wrap!(sd)),
        Matcher::new(0x707f, 0xf, |p, _| {
            panic!(format!("Unimplemented insn 'fence' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x100f, |p, _| {
            panic!(format!("Unimplemented insn 'fence.i' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2000033, |p, _| {
            panic!(format!("Unimplemented insn 'mul' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2001033, |p, _| {
            panic!(format!("Unimplemented insn 'mulh' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2002033, |p, _| {
            panic!(format!("Unimplemented insn 'mulhsu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2003033, |p, _| {
            panic!(format!("Unimplemented insn 'mulhu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2004033, |p, _| {
            panic!(format!("Unimplemented insn 'div' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2005033, |p, _| {
            panic!(format!("Unimplemented insn 'divu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2006033, |p, _| {
            panic!(format!("Unimplemented insn 'rem' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2007033, |p, _| {
            panic!(format!("Unimplemented insn 'remu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x200003b, |p, _| {
            panic!(format!("Unimplemented insn 'mulw' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x200403b, |p, _| {
            panic!(format!("Unimplemented insn 'divw' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x200503b, |p, _| {
            panic!(format!("Unimplemented insn 'divuw' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x200603b, |p, _| {
            panic!(format!("Unimplemented insn 'remw' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x200703b, |p, _| {
            panic!(format!("Unimplemented insn 'remuw' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x202f, |p, _| {
            panic!(format!("Unimplemented insn 'amoadd.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x2000202f, |p, _| {
            panic!(format!("Unimplemented insn 'amoxor.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x4000202f, |p, _| {
            panic!(format!("Unimplemented insn 'amoor.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x6000202f, |p, _| {
            panic!(format!("Unimplemented insn 'amoand.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x8000202f, |p, _| {
            panic!(format!("Unimplemented insn 'amomin.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0xa000202f, |p, _| {
            panic!(format!("Unimplemented insn 'amomax.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0xc000202f, |p, _| {
            panic!(format!("Unimplemented insn 'amominu.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0xe000202f, |p, _| {
            panic!(format!("Unimplemented insn 'amomaxu.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x800202f, |p, _| {
            panic!(format!("Unimplemented insn 'amoswap.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf9f0707f, 0x1000202f, |p, _| {
            panic!(format!("Unimplemented insn 'lr.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x1800202f, |p, _| {
            panic!(format!("Unimplemented insn 'sc.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x302f, |p, _| {
            panic!(format!("Unimplemented insn 'amoadd.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x2000302f, |p, _| {
            panic!(format!("Unimplemented insn 'amoxor.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x4000302f, |p, _| {
            panic!(format!("Unimplemented insn 'amoor.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x6000302f, |p, _| {
            panic!(format!("Unimplemented insn 'amoand.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x8000302f, |p, _| {
            panic!(format!("Unimplemented insn 'amomin.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0xa000302f, |p, _| {
            panic!(format!("Unimplemented insn 'amomax.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0xc000302f, |p, _| {
            panic!(format!("Unimplemented insn 'amominu.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0xe000302f, |p, _| {
            panic!(format!("Unimplemented insn 'amomaxu.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x800302f, |p, _| {
            panic!(format!("Unimplemented insn 'amoswap.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf9f0707f, 0x1000302f, |p, _| {
            panic!(format!("Unimplemented insn 'lr.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xf800707f, 0x1800302f, |p, _| {
            panic!(format!("Unimplemented insn 'sc.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xffffffff, 0x73, wrap!(ecall)),
        Matcher::new(0xffffffff, 0x100073, |p, _| {
            panic!(format!("Unimplemented insn 'ebreak' at {:x}", p.pc()))
        }),
        Matcher::new(0xffffffff, 0x200073, |p, _| {
            panic!(format!("Unimplemented insn 'uret' at {:x}", p.pc()))
        }),
        Matcher::new(0xffffffff, 0x10200073, |p, _| {
            panic!(format!("Unimplemented insn 'sret' at {:x}", p.pc()))
        }),
        Matcher::new(0xffffffff, 0x30200073, wrap!(mret)),
        Matcher::new(0xffffffff, 0x7b200073, |p, _| {
            panic!(format!("Unimplemented insn 'dret' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe007fff, 0x12000073, |p, _| {
            panic!(format!("Unimplemented insn 'sfence.vma' at {:x}", p.pc()))
        }),
        Matcher::new(0xffffffff, 0x10500073, |p, _| {
            panic!(format!("Unimplemented insn 'wfi' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x1073, wrap!(csrrw)),
        Matcher::new(0x707f, 0x2073, wrap!(csrrs)),
        Matcher::new(0x707f, 0x3073, |p, _| {
            panic!(format!("Unimplemented insn 'csrrc' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x5073, wrap!(csrrwi)),
        Matcher::new(0x707f, 0x6073, |p, _| {
            panic!(format!("Unimplemented insn 'csrrsi' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x7073, |p, _| {
            panic!(format!("Unimplemented insn 'csrrci' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x53, |p, _| {
            panic!(format!("Unimplemented insn 'fadd.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x8000053, |p, _| {
            panic!(format!("Unimplemented insn 'fsub.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x10000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmul.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x18000053, |p, _| {
            panic!(format!("Unimplemented insn 'fdiv.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x20000053, |p, _| {
            panic!(format!("Unimplemented insn 'fsgnj.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x20001053, |p, _| {
            panic!(format!("Unimplemented insn 'fsgnjn.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x20002053, |p, _| {
            panic!(format!("Unimplemented insn 'fsgnjx.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x28000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmin.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x28001053, |p, _| {
            panic!(format!("Unimplemented insn 'fmax.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0x58000053, |p, _| {
            panic!(format!("Unimplemented insn 'fsqrt.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x2000053, |p, _| {
            panic!(format!("Unimplemented insn 'fadd.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0xa000053, |p, _| {
            panic!(format!("Unimplemented insn 'fsub.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x12000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmul.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x1a000053, |p, _| {
            panic!(format!("Unimplemented insn 'fdiv.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x22000053, |p, _| {
            panic!(format!("Unimplemented insn 'fsgnj.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x22001053, |p, _| {
            panic!(format!("Unimplemented insn 'fsgnjn.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x22002053, |p, _| {
            panic!(format!("Unimplemented insn 'fsgnjx.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2a000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmin.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2a001053, |p, _| {
            panic!(format!("Unimplemented insn 'fmax.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0x40100053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.s.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0x42000053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.d.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0x5a000053, |p, _| {
            panic!(format!("Unimplemented insn 'fsqrt.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x6000053, |p, _| {
            panic!(format!("Unimplemented insn 'fadd.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0xe000053, |p, _| {
            panic!(format!("Unimplemented insn 'fsub.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x16000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmul.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00007f, 0x1e000053, |p, _| {
            panic!(format!("Unimplemented insn 'fdiv.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x26000053, |p, _| {
            panic!(format!("Unimplemented insn 'fsgnj.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x26001053, |p, _| {
            panic!(format!("Unimplemented insn 'fsgnjn.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x26002053, |p, _| {
            panic!(format!("Unimplemented insn 'fsgnjx.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2e000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmin.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0x2e001053, |p, _| {
            panic!(format!("Unimplemented insn 'fmax.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0x40300053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.s.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0x46000053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.q.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0x42300053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.d.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0x46100053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.q.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0x5e000053, |p, _| {
            panic!(format!("Unimplemented insn 'fsqrt.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0xa0000053, |p, _| {
            panic!(format!("Unimplemented insn 'fle.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0xa0001053, |p, _| {
            panic!(format!("Unimplemented insn 'flt.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0xa0002053, |p, _| {
            panic!(format!("Unimplemented insn 'feq.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0xa2000053, |p, _| {
            panic!(format!("Unimplemented insn 'fle.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0xa2001053, |p, _| {
            panic!(format!("Unimplemented insn 'flt.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0xa2002053, |p, _| {
            panic!(format!("Unimplemented insn 'feq.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0xa6000053, |p, _| {
            panic!(format!("Unimplemented insn 'fle.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0xa6001053, |p, _| {
            panic!(format!("Unimplemented insn 'flt.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfe00707f, 0xa6002053, |p, _| {
            panic!(format!("Unimplemented insn 'feq.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc0000053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.w.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc0100053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.wu.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc0200053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.l.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc0300053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.lu.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0707f, 0xe0000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmv.x.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0707f, 0xe0001053, |p, _| {
            panic!(format!("Unimplemented insn 'fclass.s' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc2000053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.w.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc2100053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.wu.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc2200053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.l.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc2300053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.lu.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0707f, 0xe2000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmv.x.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0707f, 0xe2001053, |p, _| {
            panic!(format!("Unimplemented insn 'fclass.d' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc6000053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.w.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc6100053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.wu.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc6200053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.l.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xc6300053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.lu.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0707f, 0xe6000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmv.x.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0707f, 0xe6001053, |p, _| {
            panic!(format!("Unimplemented insn 'fclass.q' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd0000053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.s.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd0100053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.s.wu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd0200053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.s.l' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd0300053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.s.lu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0707f, 0xf0000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmv.w.x' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd2000053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.d.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd2100053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.d.wu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd2200053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.d.l' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd2300053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.d.lu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0707f, 0xf2000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmv.d.x' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd6000053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.q.w' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd6100053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.q.wu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd6200053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.q.l' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0007f, 0xd6300053, |p, _| {
            panic!(format!("Unimplemented insn 'fcvt.q.lu' at {:x}", p.pc()))
        }),
        Matcher::new(0xfff0707f, 0xf6000053, |p, _| {
            panic!(format!("Unimplemented insn 'fmv.q.x' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x2007, |p, _| {
            panic!(format!("Unimplemented insn 'flw' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x3007, |p, _| {
            panic!(format!("Unimplemented insn 'fld' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x4007, |p, _| {
            panic!(format!("Unimplemented insn 'flq' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x2027, |p, _| {
            panic!(format!("Unimplemented insn 'fsw' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x3027, |p, _| {
            panic!(format!("Unimplemented insn 'fsd' at {:x}", p.pc()))
        }),
        Matcher::new(0x707f, 0x4027, |p, _| {
            panic!(format!("Unimplemented insn 'fsq' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x43, |p, _| {
            panic!(format!("Unimplemented insn 'fmadd.s' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x47, |p, _| {
            panic!(format!("Unimplemented insn 'fmsub.s' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x4b, |p, _| {
            panic!(format!("Unimplemented insn 'fnmsub.s' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x4f, |p, _| {
            panic!(format!("Unimplemented insn 'fnmadd.s' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x2000043, |p, _| {
            panic!(format!("Unimplemented insn 'fmadd.d' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x2000047, |p, _| {
            panic!(format!("Unimplemented insn 'fmsub.d' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x200004b, |p, _| {
            panic!(format!("Unimplemented insn 'fnmsub.d' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x200004f, |p, _| {
            panic!(format!("Unimplemented insn 'fnmadd.d' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x6000043, |p, _| {
            panic!(format!("Unimplemented insn 'fmadd.q' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x6000047, |p, _| {
            panic!(format!("Unimplemented insn 'fmsub.q' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x600004b, |p, _| {
            panic!(format!("Unimplemented insn 'fnmsub.q' at {:x}", p.pc()))
        }),
        Matcher::new(0x600007f, 0x600004f, |p, _| {
            panic!(format!("Unimplemented insn 'fnmadd.q' at {:x}", p.pc()))
        }),
    ]
}
