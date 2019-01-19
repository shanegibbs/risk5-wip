use super::*;
use crate::memory::*;

struct BincodeReader {
    // reader: BufReader<io::Stdin>,
}

impl BincodeReader {
    fn new() -> Self {
        // let reader = BufReader::new(io::stdin());
        BincodeReader {} // { reader }
    }
}

impl Iterator for BincodeReader {
    type Item = LogTuple;

    fn next(&mut self) -> Option<LogTuple> {
        use bincode;

        let val = match bincode::deserialize_from(&mut io::stdin()) {
            Ok(n) => n,
            Err(e) => {
                match *e {
                    bincode::ErrorKind::Io(ref e) => {
                        if e.kind() == io::ErrorKind::UnexpectedEof {
                            return None;
                        }
                    }
                    _ => (),
                }
                error!("Failed to read log tuple: {}", e);
                panic!("Failed to read log tuple");
            }
        };
        trace!("Read {:?}", val);

        Some(val)
    }
}

fn format_diff(expected: u64, actual: u64) -> String {
    format!(
        "Was:      0x{:016x} {:064b}\nExpected: 0x{:016x} {:064b}",
        actual, actual, expected, expected
    )
}

pub fn run() -> Result<(), io::Error> {
    let matchers = crate::build_matchers::<ByteMap>();

    let dtb = crate::load_dtb();

    let mem = ByteMap::new();
    let mut cpu = Processor::new(0x1000, mem);
    let mut last_insn: Option<Insn> = None;

    info!("Initial checks");

    for (step, log_tuple) in BincodeReader::new().enumerate() {
        let LogTuple {
            line,
            state,
            insn,
            mems,
        } = log_tuple;
        // trace!("{:?}", state);

        // stop if required
        {
            use std::env;
            if let Ok(val) = env::var("STOP_AT") {
                let current_step = format!("{}", step);
                if val == current_step {
                    return Ok(());
                }
            }
        }

        let mut fail = false;

        /* for i in 0..8 {
            let offset = (i * 4) as usize;
            debug!("0x{:16x} 0x{:16x} 0x{:16x} 0x{:16x}", cpu.regs.get(offset), cpu.regs.get(offset + 1), cpu.regs.get(offset + 2), cpu.regs.get(offset + 3));
        } */

        // validate current state

        macro_rules! fail_on {
            ($name:expr, $expected:expr, $actual:expr) => {
                if $expected != $actual {
                    error!(
                        "Fail check on {}.\n{}",
                        $name,
                        format_diff($expected, $actual)
                    );
                    fail = true;
                }
            };
        }

        /* macro_rules! warn_on {
            ($name:expr, $expected:expr, $actual:expr) => {
                let val = u64::from_str_radix(&$expected[2..], 16).expect($name);
                if val != $actual {
                    warn!(
                        "Fail {} check. Was: 0x{:016x}, expected: {}",
                        $name, $actual, $expected
                    );
                }
            };
        } */

        if cpu.pc() != state.pc {
            error!(
                "Fail pc check. Was: 0x{:016x}, expected: {}",
                cpu.pc(),
                state.pc
            );
            fail = true;
        }

        fail_on!("prv", state.prv, cpu.csrs().prv);
        fail_on!("mepc", state.mepc, cpu.csrs().mepc);
        fail_on!("mcause", state.mcause, cpu.csrs().mcause);
        fail_on!("mscratch", state.mscratch, cpu.csrs().mscratch);
        fail_on!("mtvec", state.mtvec, cpu.csrs().mtvec);
        // fail_on!("mideleg", state.mideleg, cpu.csrs().mideleg);

        {
            if state.mstatus != cpu.csrs().mstatus.val() {
                use crate::bitfield::Mstatus;
                let mstatus_expected = Mstatus::new_with_val(state.mstatus);
                error!(
                    "Fail mstatus check\n{}\nWas:      {:?}\nExpected: {:?}",
                    format_diff(state.mstatus, cpu.csrs().mstatus.val()),
                    cpu.csrs().mstatus,
                    mstatus_expected
                );
                fail = true;
            }
        }

        for (i, reg) in state.xregs.iter().enumerate() {
            let actual = cpu.regs.get(i);
            if *reg != actual {
                let msg = format!("Fail reg check on 0x{:02x} ({})\nWas:      0x{:016x} {:064b} \nExpected: 0x{:016x} {:064b}",
                    i, regs::REG_NAMES[i], actual, actual, reg, reg);
                error!("{}", msg);
                fail = true;
            }
        }

        // if cpu.fake_mem().queue_size() != 0 {
        //     // if mem.addr != "0x80009000" && mem.addr != "0x80009008" {}
        //     warn!("Memory operations still queued");
        //     // fail = true;
        // }
        cpu.mmu_mut().mem_mut().clear();
        crate::write_reset_vec(cpu.mmu_mut().mem_mut(), 0x80000000, &dtb);

        if fail {
            let last_insn = last_insn.expect("last_insn");
            error!("debug info - step: {}", step - 1);
            error!("debug info - PC:   {}", last_insn.pc);
            error!("debug info - Insn: {}", last_insn.desc);
            error!("debug info - Line: {}", line);
            panic!("Failed checks");
        }

        // load up transactions

        if step % 10000 == 0 {
            warn!("--- Begin step {} ({}) ---", step, line);
        }
        info!("--- Begin step {} ({}) ---", step, line);

        debug!("{:?}", insn);

        // let insn_pc = u64::from_str_radix(&insn.pc[2..], 16).expect("pc");
        // let insn_bits = u32::from_str_radix(&insn.bits[2..], 16).expect("insn bits");

        // if let Some(mem) = load {
        //     trace!("Load {:?}", mem);
        //     cpu.fake_mem().push_read(mem);
        // }

        // if let Some(mem) = store {
        //     trace!("Store {:?}", mem);
        //     cpu.fake_mem().push_write(mem);
        // }

        // cpu.fake_mem()
        //     .push_read(FakeMemoryItem::Word(insn_pc, insn_bits));

        trace!("Have {} mems", mems.len());
        for mem in mems {
            trace!("{:?}", mem);
            use crate::Memory;
            cpu.mmu_mut().mem_mut().write_b(mem.addr, mem.value as u8);
        }

        for (addr, value) in &cpu.mmu().mem().data {
            if *addr >= 0x4096 {
                trace!("Have 0x{:x}: 0x{:x}", addr, value);
            }
        }

        cpu.step(&matchers);

        last_insn = Some(insn);
    }

    Ok(())
}
