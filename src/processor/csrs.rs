use crate::bitfield::Mstatus;
use crate::bitfield::Satp;
use crate::insns::Trap;
use std::fmt;

pub struct Csrs {
    prv: u64,

    pub(crate) mstatus: Mstatus,
    pub(crate) medeleg: u64,
    pub(crate) mideleg: u64,
    pub(crate) mtvec: u64,
    pub(crate) mepc: u64,
    pub(crate) mtval: u64,
    pub(crate) mcause: u64,
    pub(crate) mscratch: u64,
    pub(crate) misa: u64,
    pub(crate) mcounteren: u64,
    pub(crate) mie: u64,
    pub(crate) mip: u64,

    pub(crate) sedeleg: u64,
    pub(crate) sideleg: u64,
    pub(crate) sie: u64,
    pub(crate) stvec: u64,
    pub(crate) scounteren: u64,
    pub(crate) sscratch: u64,
    pub(crate) sepc: u64,
    pub(crate) scause: u64,
    pub(crate) stval: u64,
    pub(crate) sip: u64,
    pub(crate) satp: Satp,
}

pub enum PostSetOp {
    None,
    SetMemMode(SetMemMode),
    UpdateMmuPrv,
}

pub struct SetMemMode {
    pub mode: u64,
    pub asid: u64,
    pub ppn: u64,
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
        let mut mstatus = Mstatus::default();
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
            satp: Default::default(),
        }
    }

    #[inline(always)]
    pub fn set<T: Into<usize>>(&mut self, i: T, v: u64) -> PostSetOp {
        let i = i.into();
        debug!("Setting CSR 0x{:x} to 0x{:x} with prv {}", i, v, self.prv);
        match i {
            MTVEC => self.mtvec = v,
            MSTATUS => {
                debug!("Setting mstatus to 0x{:x}", v);
                let mut mstatus: Mstatus = v.into();
                mstatus.set_supervisor_xlen(2);
                mstatus.set_user_xlen(2);
                self.mstatus = mstatus;
                return PostSetOp::UpdateMmuPrv;
            }
            MEPC => self.mepc = v & !0x1,
            MIP => self.mip = v,
            MIE => self.mie = v,
            MEDELEG => self.medeleg = v,
            MIDELEG => self.mideleg = v,
            MSCRATCH => self.mscratch = v,
            MCOUNTEREN => self.mcounteren = v,

            SSTATUS => self.mstatus.set_bits(v),
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
                let satp: Satp = v.into();
                let mode = satp.mode();
                let asid = satp.asid();
                let ppn = satp.ppn();
                self.satp = satp;

                return PostSetOp::SetMemMode(SetMemMode {
                    mode: mode,
                    asid: asid,
                    ppn: ppn,
                });
            }

            i => warn!("unimplemented Csrs.set 0x{:x}", i),
        }
        PostSetOp::None
    }

    #[inline(always)]
    pub fn get<T: Into<usize>>(&self, i: T) -> Result<u64, Trap> {
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

            SSTATUS => self.mstatus.val_for_prv(self.prv),
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
            SATP => (&self.satp).into(),

            i => {
                warn!("unimplemented Csrs.get 0x{:x}. Triggering trap", i);
                return Err(Trap::illegal_insn());
            }
        });
    }

    pub fn prv(&self) -> u64 {
        self.prv
    }

    pub(super) fn set_prv(&mut self, prv: u64) {
        self.prv = prv
    }
}

impl fmt::Debug for Csrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Csrs")
    }
}

use crate::logrunner::State;
impl Into<Csrs> for &State {
    fn into(self) -> Csrs {
        Csrs {
            prv: self.prv,
            mstatus: self.mstatus.into(),
            medeleg: self.medeleg,
            mideleg: self.mideleg,
            mtvec: self.mtvec,
            mepc: self.mepc,
            mtval: 0, // self.mtval,
            mcause: self.mcause,
            mscratch: self.mscratch,
            misa: 0x8000000000141101,
            mcounteren: self.mcounteren,
            mie: self.mie,
            mip: self.mip,
            sedeleg: 0, // self.sedeleg,
            sideleg: 0, // self.sideleg,
            sie: 0,     // self.sie,
            stvec: self.stvec,
            scounteren: self.scounteren,
            sscratch: self.sscratch,
            sepc: self.sepc,
            scause: self.scause,
            stval: self.stval,
            sip: 0, // self.sip,
            satp: self.satp.into(),
        }
    }
}
