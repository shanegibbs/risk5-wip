use crate::mstatus::Mstatus;
use std::fmt;

pub struct Csrs {
    pub prv: u64,

    pub mstatus: Mstatus,
    pub medeleg: u64,
    pub mideleg: u64,
    pub mtvec: u64,
    pub mepc: u64,
    pub mtval: u64,
    pub mcause: u64,
    pub mscratch: u64,
    pub misa: u64,
    pub mcounteren: u64,
    pub mie: u64,
    pub mip: u64,

    pub sstatus: u64,
    pub sedeleg: u64,
    pub sideleg: u64,
    pub sie: u64,
    pub stvec: u64,
    pub scounteren: u64,
    pub sscratch: u64,
    pub sepc: u64,
    pub scause: u64,
    pub stval: u64,
    pub sip: u64,
    pub satp: u64,
}

// Supervisor

// Supervisor Trap Setup
const SSTATUS: usize = 0x100;
const SEDELEG: usize = 0x102;
const SIDELEG: usize = 0x103;
const SIE: usize = 0x104;
const STVEC: usize = 0x105;
const SCOUNTEREN: usize = 0x106;

// Supervisor Trap Handling
const SSCRATCH: usize = 0x140;
const SEPC: usize = 0x141;
const SCAUSE: usize = 0x142;
const STVAL: usize = 0x143;
const SIP: usize = 0x144;

// Supervisor Protection and Translation
const SATP: usize = 0x180;

// Machine CSRs

// Machine Information Registers
const MHARTID: usize = 0xf14;

// Machine Trap Setup
const MSTATUS: usize = 0x300;
const MISA: usize = 0x301;
const MEDELEG: usize = 0x302;
const MIDELEG: usize = 0x303;
const MIE: usize = 0x304;
const MTVEC: usize = 0x305;
const MCOUNTEREN: usize = 0x306;

// Machine Trap Handling
const MSCRATCH: usize = 0x340;
const MEPC: usize = 0x341;
const MCAUSE: usize = 0x342;
const MTVAL: usize = 0x343;
const MIP: usize = 0x344;

impl Csrs {
    pub fn new() -> Self {
        let mut mstatus = Mstatus::new();
        mstatus.set_supervisor_xlen(2);
        mstatus.set_user_xlen(2);

        Csrs {
            prv: 3,
            mstatus: mstatus,
            medeleg: 0,
            mideleg: 0,
            mtvec: 0,
            mepc: 0,
            mtval: 0,
            mcause: 0,
            mscratch: 0,
            misa: 0x8000000000141101,
            mcounteren: 0,
            mie: 0,
            mip: 0,

            sstatus: 0,
            sedeleg: 0,
            sideleg: 0,
            sie: 0,
            stvec: 0,
            scounteren: 0,
            sscratch: 0,
            sepc: 0,
            scause: 0,
            stval: 0,
            sip: 0,
            satp: 0,
        }
    }

    #[inline(always)]
    pub fn set<T: Into<usize>>(&mut self, i: T, v: u64) {
        let i = i.into();
        debug!("Setting CSR 0x{:x} to 0x{:x} with prv {}", i, v, self.prv);
        match i {
            MTVEC => self.mtvec = v,
            MSTATUS => {
                debug!("Setting mstatus to 0x{:x}", v);
                let mut mstatus = Mstatus::new_with_val(v);
                mstatus.set_supervisor_xlen(2);
                mstatus.set_user_xlen(2);
                self.mstatus = mstatus;
            }
            MEPC => self.mepc = v & !0x1,
            MIP => self.mip = v,
            MIE => self.mie = v,
            MEDELEG => self.medeleg = v,
            MIDELEG => self.mideleg = v,
            MSCRATCH => self.mscratch = v,
            MCOUNTEREN => self.mcounteren = v,

            SSTATUS => self.sstatus = v,
            SEDELEG => self.sedeleg = v,
            SIDELEG => self.sideleg = v,
            SIE => self.sie = v,
            STVEC => self.stvec = v,
            SCOUNTEREN => self.scounteren = v,
            SSCRATCH => self.sscratch = v,
            SEPC => self.sepc = v,
            SCAUSE => self.scause = v,
            STVAL => self.stval = v,
            SIP => self.sip = v,
            SATP => {
                if v != 0 {
                    panic!("unimplemented set SATP to 0x{:x}", v)
                }
            }

            i => warn!("unimplemented Csrs.set 0x{:x}", i),
        }
    }

    #[inline(always)]
    pub fn get<T: Into<usize>>(&self, i: T) -> Result<u64, u64> {
        let i = i.into();
        trace!("Getting CSR 0x{:x} with prv {}", i, self.prv);
        return Ok(match i {
            MHARTID => 0,

            MSTATUS => self.mstatus.val(),
            MISA => self.misa,
            MIP => self.mip,
            MIE => self.mie,
            MEDELEG => self.medeleg,
            MIDELEG => self.mideleg,
            MCOUNTEREN => self.mcounteren,
            MTVEC => self.mtvec,
            MEPC => self.mepc,
            MTVAL => self.mtval,
            MSCRATCH => self.mscratch,
            MCAUSE => self.mcause,

            SSTATUS => self.sstatus,
            SEDELEG => self.sedeleg,
            SIDELEG => self.sideleg,
            SIE => self.sie,
            STVEC => self.stvec,
            SCOUNTEREN => self.scounteren,
            SSCRATCH => self.sscratch,
            SEPC => self.sepc,
            SCAUSE => self.scause,
            STVAL => self.stval,
            SIP => self.sip,
            SATP => self.satp,

            i => {
                warn!("unimplemented Csrs.get 0x{:x}. Triggering trap", i);
                return Err(2); // Illegal instruction
            }
        });
    }
}

impl fmt::Debug for Csrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Csrs")
    }
}
