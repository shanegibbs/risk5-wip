#[macro_use] extern crate log;
extern crate pretty_env_logger;
extern crate elf;

mod insn;
mod opcodes;
mod elf_loader;
mod mmu;
mod csrs;

use std::fs::File;
use std::io::Read;

pub use insn::Instruction;

pub fn main() {
	pretty_env_logger::init();

    let mut mem = mmu::Memory::new(15);

    // file offset, mem offset, size
    let (entry, sections) = elf_loader::read_program_segments();

    let mut elf = File::open("../bins/bbl/bbl").unwrap();
    let mut file_bytes = vec![];
    let _read_file_size = elf.read_to_end(&mut file_bytes).unwrap();

    debug!("Loading ELF");
    for (f_offset, m_offset, size) in sections {
        mem.add_block(m_offset as usize, size as usize);
        for i in 0..size {
            mem.write_b((m_offset + i) as usize, file_bytes[(f_offset + i) as usize]);
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
    //     0x00000297, 0x02028593, 0xf1402573, 0x0182b283, 0x00028067, 0x2000006f, 0x00000093,
    // ];

    let mut csrs = csrs::Csrs::new();
    let mut cpu = opcodes::Processor::new(entry);
    loop {
        cpu.step(&mut csrs, &mut mem);
    }

}
