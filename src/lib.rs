#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

// macro_rules! fatal {
//     ($($arg:tt)*) => (
//         error!($($arg)*);
//         panic!($($arg)*);
//     )
// }

mod bitfield;
mod elf_loader;
mod insns;
mod itypes;
pub mod logrunner;
mod matcher;
mod memory;
mod mmu;
mod processor;
mod regs;

pub use crate::insns::*;
pub(crate) use crate::matcher::{Matcher, Matchers};
use crate::memory::BlockMemory;
use crate::memory::Memory;
pub(crate) use crate::mmu::Mmu;
pub use crate::processor::Processor;
pub(crate) use crate::regs::Regs;
use std::fs::File;
use std::io::Read;

pub fn load_dtb() -> Vec<u8> {
    let mut dtb = vec![];
    let mut dtb_file = File::open("assets/dtb.bin").expect("dtb.bin");
    dtb_file.read_to_end(&mut dtb).expect("read dtb");
    dtb
}

pub fn write_reset_vec<M: Memory>(mem: &mut M, entry: u64, dtb: &[u8]) {
    //  auipc   t0, 0x0
    //  addi    a1, t0, 32
    //  csrr    a0, mhartid
    //  ld      t0, 24(t0)
    //  jr      t0

    let reset_vec_addr = 0x1000;
    let reset_vec_size = 8;
    mem.write_w(reset_vec_addr, 0x297);
    mem.write_w(
        reset_vec_addr + 4,
        0x28593 + ((reset_vec_size * 4) << 20) as u32,
    );
    mem.write_w(reset_vec_addr + 8, 0xf140_2573);
    mem.write_w(reset_vec_addr + 12, 0x0182_b283);
    mem.write_w(reset_vec_addr + 16, 0x28067);

    mem.write_w(reset_vec_addr + 20, 0x0);
    mem.write_w(reset_vec_addr + 24, entry as u32);
    mem.write_w(reset_vec_addr + 28, (entry >> 32) as u32);
    for (i, b) in dtb.iter().enumerate() {
        mem.write_b(reset_vec_addr + 32 + i as u64, *b);
    }
}

pub fn build_memory() -> BlockMemory {
    let mut mem = BlockMemory::new(15);

    mem.add_block(0x8000_0000, 2048 * 1024 * 1024);

    // dummy clint
    mem.add_block(0x200_0000, 0xc000);

    let reset_vec_addr = 0x1000;
    mem.add_block(reset_vec_addr, 2048);

    use std::env;
    let filename = env::var("BIN").unwrap_or_else(|_| "assets/bbl".into());

    let (entry, sections) = elf_loader::read_program_segments(&filename);
    let mut elf = File::open(&filename).unwrap();
    let mut file_bytes = vec![];
    let _read_file_size = elf.read_to_end(&mut file_bytes).unwrap();

    debug!("Loading ELF");
    for (f_offset, m_offset, size) in sections {
        debug!("Loading 0x{:x} bytes @ 0x{:x}", size, m_offset);
        for i in 0..size {
            mem.write_b(m_offset + i, file_bytes[(f_offset + i) as usize]);
        }
    }

    let dtb = load_dtb();
    write_reset_vec(&mut mem, entry, &dtb);

    mem
}

pub fn risk5_main() {
    pretty_env_logger::init();
    // logrunner::logger::init().unwrap();

    let mut cpu = Processor::new(build_memory());
    let matchers = &mut build_matchers();

    use std::time::SystemTime;
    let start = SystemTime::now();
    let mut mark = SystemTime::now();

    use std::sync::{Arc, RwLock};
    let trigger = Arc::new(RwLock::new(false));

    {
        use std::io::stdin;
        use std::thread::spawn;
        let trigger = trigger.clone();

        spawn(move || {
            let mut buf = String::new();
            loop {
                stdin().read_line(&mut buf).expect("stdin");
                let line = buf.trim_end();
                println!("{}", line);

                if line == "" {
                    let mut i = trigger.write().expect("write lock");
                    *i = !*i;
                    warn!("trigger={}", i);
                }

                buf.clear();
            }
        });
    }

    const STEP_SIZE: usize = 10_000_000;

    let mut counter = 0;
    let mut real_trigger = false;

    loop {
        if cpu.pc() == 0xffffffe0001524b0 {
            warn!("entering arch_cpu_idle()");
        }
        // if cpu.pc() == 0xffffffe000151f48 {
        //     error!("here");
        //     break;
        // }

        cpu.step(matchers);
        // calls[idx] += 1;
        // let matcher = matchers.remove(idx).expect("used insn");
        // matchers.push_front(matcher);

        counter += 1;
        trace!("--- Step {} ---", counter);

        if counter % 1000 == 0 {
            cpu.handle_interrupt();

            real_trigger = *trigger.read().expect("read lock");
            cpu.trigger = real_trigger;

            if counter % STEP_SIZE == 0 {
                // if counter >= 50_000_000 {
                //     break;
                // }

                let d = SystemTime::now().duration_since(mark).expect("time");
                let in_ms = d.as_secs() * 1000 + d.subsec_nanos() as u64 / 1_000_000;
                let in_sec = (in_ms as f32) / 1000f32;
                let speed = (STEP_SIZE as f32) / in_sec;
                // warn!(
                //     "--- Step {}mil --- pc=0x{:x} @ {} MHz",
                //     counter / 1_000_000,
                //     cpu.pc(),
                //     speed / 1_000_000.0
                // );
                mark = SystemTime::now();

                // if counter == 180_000_000 {
                //     error!("too slow");
                //     panic!("too slow");
                // }
            }
        }

        // // let _fromhost = cpu.mmu_mut().bare_mut().read_d(0x80009000);
        // let tohost = cpu.mmu_mut().bare_mut().read_d(0x8000_9008);

        // if tohost > 0 {
        //     let ch = tohost as u8;
        //     output = format!("{}{}", output, ch as char);
        //     use std::io::{self, Write};
        //     write!(io::stderr(), "{}", ch as char).expect("stderr write");
        //     info!("tohost '{}'", ch as char);
        //     cpu.mmu_mut().bare_mut().write_d(0x8000_9008, 0);
        // }
    }

    // let d = SystemTime::now().duration_since(start).expect("time");
    // let in_ms = d.as_secs() * 1000 + d.subsec_nanos() as u64 / 1_000_000;
    // let in_sec = (in_ms as f32) / 1000f32;
    // let speed = (counter as f32) / in_sec;
    // println!(
    //     "Executed {}m insns @ {} MHz",
    //     counter / 1_000_000,
    //     speed / 1_000_000.0
    // );

    // matchers.print();
}

pub fn build_matchers<M: Memory>() -> Matchers<M> {
    macro_rules! wrap {
        ($f:path) => {
            |p, i| {
                let a = i;
                let i = i.into();
                debug!("> exec 0x{:x} 0x{:08x} {} {}", p.pc(), a, stringify!($f), i);
                $f(p, i)
            }
        };
    }

    macro_rules! wrap_no_arg {
        ($f:path) => {
            |p, i| {
                debug!("> exec 0x{:x} {} {}", p.pc(), stringify!($f), i);
                $f(p)
            }
        };
    }

    macro_rules! noimpl {
        ($insn:expr) => {
            |p, i| {
                error!(
                    "Unimplented insn {} (0x{:x}) at 0x{:x}",
                    stringify!($insn),
                    i,
                    p.pc()
                );
                p.advance_pc();
                panic!("no insn impl");
            }
        };
    }

    use crate::insns::csr;
    use crate::insns::mem;

    Matchers::new(vec![
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
        Matcher::new(0x707f, 0x13, wrap!(comp::imm<M, comp::Add>)),
        Matcher::new(0xfc00707f, 0x1013, wrap!(comp::imm<M, comp::Sll>)),
        Matcher::new(0x707f, 0x2013, wrap!(comp::imm<M, comp::Slt>)),
        Matcher::new(0x707f, 0x3013, wrap!(comp::immu<M, comp::Slt>)),
        Matcher::new(0x707f, 0x4013, wrap!(comp::imm<M, comp::Xor>)),
        Matcher::new(0xfc00707f, 0x5013, wrap!(comp::immu<M, comp::Srl>)),
        Matcher::new(0xfc00707f, 0x40005013, wrap!(comp::imm<M, comp::Sra>)),
        Matcher::new(0x707f, 0x6013, wrap!(comp::imm<M, comp::Or>)),
        Matcher::new(0x707f, 0x7013, wrap!(comp::imm<M, comp::And>)),
        Matcher::new(0xfe00707f, 0x33, wrap!(comp::reg<M, comp::Add>)),
        Matcher::new(0xfe00707f, 0x40000033, wrap!(comp::reg<M, comp::Sub>)),
        Matcher::new(0xfe00707f, 0x1033, wrap!(comp::reg<M, comp::Sll>)),
        Matcher::new(0xfe00707f, 0x2033, wrap!(comp::reg<M, comp::Slt>)),
        Matcher::new(0xfe00707f, 0x3033, wrap!(comp::regu<M, comp::Slt>)),
        Matcher::new(0xfe00707f, 0x4033, wrap!(comp::reg<M, comp::Xor>)),
        Matcher::new(0xfe00707f, 0x5033, wrap!(comp::regu<M, comp::Srl>)),
        Matcher::new(0xfe00707f, 0x40005033, wrap!(comp::reg<M, comp::Sra>)),
        Matcher::new(0xfe00707f, 0x6033, wrap!(comp::reg<M, comp::Or>)),
        Matcher::new(0xfe00707f, 0x7033, wrap!(comp::reg<M, comp::And>)),
        Matcher::new(0x707f, 0x1b, wrap!(comp::immw<M, comp::Add>)),
        Matcher::new(0xfe00707f, 0x101b, wrap!(comp::immw<M, comp::Sll>)),
        Matcher::new(0xfe00707f, 0x501b, wrap!(comp::immwu<M, comp::Srl>)),
        Matcher::new(0xfe00707f, 0x4000501b, wrap!(comp::immw<M, comp::Sra>)),
        Matcher::new(0xfe00707f, 0x3b, wrap!(comp::regw<M, comp::Add>)),
        Matcher::new(0xfe00707f, 0x4000003b, wrap!(comp::regw<M, comp::Sub>)),
        Matcher::new(0xfe00707f, 0x103b, wrap!(comp::regw<M, comp::Sll>)),
        Matcher::new(0xfe00707f, 0x503b, wrap!(comp::regw<M, comp::Srl>)),
        Matcher::new(0xfe00707f, 0x4000503b, wrap!(comp::regw<M, comp::Sra>)),
        Matcher::new(0x707f, 0x3, wrap!(mem::lb)),
        Matcher::new(0x707f, 0x1003, wrap!(mem::lh)),
        Matcher::new(0x707f, 0x2003, wrap!(mem::lw)),
        Matcher::new(0x707f, 0x3003, wrap!(mem::ld)),
        Matcher::new(0x707f, 0x4003, wrap!(mem::lbu)),
        Matcher::new(0x707f, 0x5003, wrap!(mem::lhu)),
        Matcher::new(0x707f, 0x6003, wrap!(mem::lwu)),
        Matcher::new(0x707f, 0x23, wrap!(mem::sb)),
        Matcher::new(0x707f, 0x1023, wrap!(mem::sh)),
        Matcher::new(0x707f, 0x2023, wrap!(mem::sw)),
        Matcher::new(0x707f, 0x3023, wrap!(mem::sd)),
        Matcher::new(0x707f, 0xf, |p, _| {
            trace!("Unimplemented insn 'fence' at {:x}", p.pc());
            p.mmu_mut().flush_cache();
            p.advance_pc();
        }),
        Matcher::new(0x707f, 0x100f, |p, _| {
            trace!("Unimplemented insn 'fence.i' at {:x}", p.pc());
            p.mmu_mut().flush_cache();
            p.advance_pc();
        }),
        Matcher::new(0xfe00707f, 0x2000033, wrap!(comp::reg<M, comp::Mul>)),
        Matcher::new(0xfe00707f, 0x2001033, wrap!(comp::reg<M, comp::Mulh>)),
        Matcher::new(0xfe00707f, 0x2002033, noimpl!("mulhsu")),
        Matcher::new(0xfe00707f, 0x2003033, wrap!(comp::regu<M, comp::Mulh>)),
        Matcher::new(0xfe00707f, 0x2004033, wrap!(comp::reg<M, comp::Div>)),
        Matcher::new(0xfe00707f, 0x2005033, wrap!(comp::regu<M, comp::Div>)),
        Matcher::new(0xfe00707f, 0x2006033, wrap!(comp::reg<M, comp::Rem>)),
        Matcher::new(0xfe00707f, 0x2007033, wrap!(comp::regu<M, comp::Rem>)),
        Matcher::new(0xfe00707f, 0x200003b, wrap!(comp::regw<M, comp::Mul>)),
        Matcher::new(0xfe00707f, 0x200403b, wrap!(comp::regw<M, comp::Div>)),
        Matcher::new(0xfe00707f, 0x200503b, wrap!(comp::reguw<M, comp::Div>)),
        Matcher::new(0xfe00707f, 0x200603b, wrap!(comp::regw<M, comp::Rem>)),
        Matcher::new(0xfe00707f, 0x200703b, wrap!(comp::reguw<M, comp::Rem>)),
        Matcher::new(0xf800707f, 0x202f, wrap!(amoaddw)),
        Matcher::new(0xf800707f, 0x2000202f, noimpl!("amoxor.w")),
        Matcher::new(0xf800707f, 0x4000202f, noimpl!("amoor.w")),
        Matcher::new(0xf800707f, 0x6000202f, noimpl!("amoand.w")),
        Matcher::new(0xf800707f, 0x8000202f, noimpl!("amomin.w")),
        Matcher::new(0xf800707f, 0xa000202f, noimpl!("amomax.w")),
        Matcher::new(0xf800707f, 0xc000202f, noimpl!("amominu.w")),
        Matcher::new(0xf800707f, 0xe000202f, noimpl!("amomaxu.w")),
        Matcher::new(0xf800707f, 0x800202f, wrap!(amoswapw)),
        Matcher::new(0xf9f0707f, 0x1000202f, wrap!(lrw)),
        Matcher::new(0xf800707f, 0x1800202f, wrap!(scw)),
        Matcher::new(0xf800707f, 0x302f, wrap!(amoaddd)),
        Matcher::new(0xf800707f, 0x2000302f, noimpl!("amoxor.d")),
        Matcher::new(0xf800707f, 0x4000302f, wrap!(amoord)),
        Matcher::new(0xf800707f, 0x6000302f, wrap!(amoandd)),
        Matcher::new(0xf800707f, 0x8000302f, noimpl!("amomin.d")),
        Matcher::new(0xf800707f, 0xa000302f, noimpl!("amomax.d")),
        Matcher::new(0xf800707f, 0xc000302f, noimpl!("amominu.d")),
        Matcher::new(0xf800707f, 0xe000302f, noimpl!("amomaxu.d")),
        Matcher::new(0xf800707f, 0x800302f, wrap!(amoswapd)),
        Matcher::new(0xf9f0707f, 0x1000302f, wrap!(lrd)),
        Matcher::new(0xf800707f, 0x1800302f, wrap!(scd)),
        Matcher::new(0xffffffff, 0x73, wrap!(ecall)),
        Matcher::new(0xffffffff, 0x100073, noimpl!("ebreak")),
        Matcher::new(0xffffffff, 0x200073, noimpl!("uret")),
        Matcher::new(0xffffffff, 0x10200073, wrap!(sret)),
        Matcher::new(0xffffffff, 0x30200073, wrap!(mret)),
        Matcher::new(0xffffffff, 0x7b200073, noimpl!("dret")),
        Matcher::new(0xfe007fff, 0x12000073, |p, _| {
            trace!("Noop insn 'sfence.vma' at {:x}", p.pc());
            p.mmu_mut().flush_cache();
            p.advance_pc();
        }),
        Matcher::new(0xffffffff, 0x10500073, wrap_no_arg!(wfi)),
        Matcher::new(0x707f, 0x1073, wrap!(csr::insn<M, csr::ReadWrite>)),
        Matcher::new(0x707f, 0x2073, wrap!(csr::insn<M, csr::ReadSet>)),
        Matcher::new(0x707f, 0x3073, wrap!(csr::insn<M, csr::ReadClear>)),
        Matcher::new(0x707f, 0x5073, wrap!(csr::insn<M, csr::ReadWriteImm>)),
        Matcher::new(0x707f, 0x6073, wrap!(csr::insn<M, csr::ReadSetImm>)),
        Matcher::new(0x707f, 0x7073, wrap!(csr::insn<M, csr::ReadClearImm>)),
        // Matcher::new(0xfe00007f, 0x53, |p, _| {
        //     error!("Unimplemented insn 'fadd.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0x8000053, |p, _| {
        //     error!("Unimplemented insn 'fsub.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0x10000053, |p, _| {
        //     error!("Unimplemented insn 'fmul.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0x18000053, |p, _| {
        //     error!("Unimplemented insn 'fdiv.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x20000053, |p, _| {
        //     error!("Unimplemented insn 'fsgnj.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x20001053, |p, _| {
        //     error!("Unimplemented insn 'fsgnjn.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x20002053, |p, _| {
        //     error!("Unimplemented insn 'fsgnjx.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x28000053, |p, _| {
        //     error!("Unimplemented insn 'fmin.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x28001053, |p, _| {
        //     error!("Unimplemented insn 'fmax.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0x58000053, |p, _| {
        //     error!("Unimplemented insn 'fsqrt.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0x2000053, |p, _| {
        //     error!("Unimplemented insn 'fadd.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0xa000053, |p, _| {
        //     error!("Unimplemented insn 'fsub.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0x12000053, |p, _| {
        //     error!("Unimplemented insn 'fmul.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0x1a000053, |p, _| {
        //     error!("Unimplemented insn 'fdiv.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x22000053, |p, _| {
        //     error!("Unimplemented insn 'fsgnj.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x22001053, |p, _| {
        //     error!("Unimplemented insn 'fsgnjn.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x22002053, |p, _| {
        //     error!("Unimplemented insn 'fsgnjx.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x2a000053, |p, _| {
        //     error!("Unimplemented insn 'fmin.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x2a001053, |p, _| {
        //     error!("Unimplemented insn 'fmax.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0x40100053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.s.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0x42000053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.d.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0x5a000053, |p, _| {
        //     error!("Unimplemented insn 'fsqrt.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0x6000053, |p, _| {
        //     error!("Unimplemented insn 'fadd.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0xe000053, |p, _| {
        //     error!("Unimplemented insn 'fsub.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0x16000053, |p, _| {
        //     error!("Unimplemented insn 'fmul.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00007f, 0x1e000053, |p, _| {
        //     error!("Unimplemented insn 'fdiv.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x26000053, |p, _| {
        //     error!("Unimplemented insn 'fsgnj.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x26001053, |p, _| {
        //     error!("Unimplemented insn 'fsgnjn.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x26002053, |p, _| {
        //     error!("Unimplemented insn 'fsgnjx.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x2e000053, |p, _| {
        //     error!("Unimplemented insn 'fmin.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0x2e001053, |p, _| {
        //     error!("Unimplemented insn 'fmax.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0x40300053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.s.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0x46000053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.q.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0x42300053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.d.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0x46100053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.q.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0x5e000053, |p, _| {
        //     error!("Unimplemented insn 'fsqrt.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0xa0000053, |p, _| {
        //     error!("Unimplemented insn 'fle.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0xa0001053, |p, _| {
        //     error!("Unimplemented insn 'flt.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0xa0002053, |p, _| {
        //     error!("Unimplemented insn 'feq.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0xa2000053, |p, _| {
        //     error!("Unimplemented insn 'fle.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0xa2001053, |p, _| {
        //     error!("Unimplemented insn 'flt.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0xa2002053, |p, _| {
        //     error!("Unimplemented insn 'feq.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0xa6000053, |p, _| {
        //     error!("Unimplemented insn 'fle.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0xa6001053, |p, _| {
        //     error!("Unimplemented insn 'flt.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfe00707f, 0xa6002053, |p, _| {
        //     error!("Unimplemented insn 'feq.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc0000053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.w.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc0100053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.wu.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc0200053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.l.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc0300053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.lu.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0707f, 0xe0000053, |p, _| {
        //     error!("Unimplemented insn 'fmv.x.w' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0707f, 0xe0001053, |p, _| {
        //     error!("Unimplemented insn 'fclass.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc2000053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.w.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc2100053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.wu.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc2200053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.l.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc2300053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.lu.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0707f, 0xe2000053, |p, _| {
        //     error!("Unimplemented insn 'fmv.x.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0707f, 0xe2001053, |p, _| {
        //     error!("Unimplemented insn 'fclass.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc6000053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.w.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc6100053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.wu.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc6200053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.l.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xc6300053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.lu.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0707f, 0xe6000053, |p, _| {
        //     error!("Unimplemented insn 'fmv.x.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0707f, 0xe6001053, |p, _| {
        //     error!("Unimplemented insn 'fclass.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd0000053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.s.w' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd0100053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.s.wu' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd0200053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.s.l' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd0300053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.s.lu' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0707f, 0xf0000053, |p, _| {
        //     error!("Unimplemented insn 'fmv.w.x' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd2000053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.d.w' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd2100053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.d.wu' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd2200053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.d.l' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd2300053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.d.lu' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0707f, 0xf2000053, |p, _| {
        //     error!("Unimplemented insn 'fmv.d.x' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd6000053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.q.w' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd6100053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.q.wu' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd6200053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.q.l' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0007f, 0xd6300053, |p, _| {
        //     error!("Unimplemented insn 'fcvt.q.lu' at {:x}", p.pc())
        // }),
        // Matcher::new(0xfff0707f, 0xf6000053, |p, _| {
        //     error!("Unimplemented insn 'fmv.q.x' at {:x}", p.pc())
        // }),
        // Matcher::new(0x707f, 0x2007, |p, _| {
        //     error!("Unimplemented insn 'flw' at {:x}", p.pc())
        // }),
        // Matcher::new(0x707f, 0x3007, |p, _| {
        //     error!("Unimplemented insn 'fld' at {:x}", p.pc())
        // }),
        // Matcher::new(0x707f, 0x4007, |p, _| {
        //     error!("Unimplemented insn 'flq' at {:x}", p.pc())
        // }),
        // Matcher::new(0x707f, 0x2027, |p, _| {
        //     error!("Unimplemented insn 'fsw' at {:x}", p.pc())
        // }),
        // Matcher::new(0x707f, 0x3027, |p, _| {
        //     error!("Unimplemented insn 'fsd' at {:x}", p.pc())
        // }),
        // Matcher::new(0x707f, 0x4027, |p, _| {
        //     error!("Unimplemented insn 'fsq' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x43, |p, _| {
        //     error!("Unimplemented insn 'fmadd.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x47, |p, _| {
        //     error!("Unimplemented insn 'fmsub.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x4b, |p, _| {
        //     error!("Unimplemented insn 'fnmsub.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x4f, |p, _| {
        //     error!("Unimplemented insn 'fnmadd.s' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x2000043, |p, _| {
        //     error!("Unimplemented insn 'fmadd.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x2000047, |p, _| {
        //     error!("Unimplemented insn 'fmsub.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x200004b, |p, _| {
        //     error!("Unimplemented insn 'fnmsub.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x200004f, |p, _| {
        //     error!("Unimplemented insn 'fnmadd.d' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x6000043, |p, _| {
        //     error!("Unimplemented insn 'fmadd.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x6000047, |p, _| {
        //     error!("Unimplemented insn 'fmsub.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x600004b, |p, _| {
        //     error!("Unimplemented insn 'fnmsub.q' at {:x}", p.pc())
        // }),
        // Matcher::new(0x600007f, 0x600004f, |p, _| {
        //     error!("Unimplemented insn 'fnmadd.q' at {:x}", p.pc())
        // }),
    ])
}
