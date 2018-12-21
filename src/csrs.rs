use std::fmt;

pub struct Csrs {
    pub prv: u64,
    pub satp: u64,
    pub mstatus: u64,
    pub medeleg: u64,
    pub mideleg: u64,
    pub mtvec: u64,
    pub mepc: u64,
    pub mtval: u64,
}

// Supervisor Protection and Translation
const SATP: usize = 0x180;

// Machine Information Registers
const MHARTID: usize = 0xf14;

// Machine Trap Setup
const MSTATUS: usize = 0x300;
const MEDELEG: usize = 0x302;
const MIDELEG: usize = 0x303;
const MTVEC: usize = 0x305;

// Machine Trap Handling
const MEPC: usize = 0x341;
const MTVAL: usize = 0x343;

impl Csrs {
    pub fn new() -> Self {
        Csrs {
            prv: 3,
            satp: 0,
            mstatus: 0,
            medeleg: 0,
            mideleg: 0,
            mtvec: 0,
            mepc: 0,
            mtval: 0,
        }
    }
    pub fn set<T: Into<usize>>(&mut self, i: T, v: u64) {
        let i = i.into();
        debug!("Setting CSR 0x{:x} to 0x{:x}", i, v);
        if i == MTVEC {
            self.mtvec = v
        } else if i == SATP {
            if v != 0 {
                panic!("unimplemented set SATP to 0x{:x}", v)
            }
        } else {
            error!("unimplemented Csrs.set 0x{:x}", i)
        }
    }
    pub fn get<T: Into<usize>>(&self, i: T) -> Result<u64, ()> {
        let i = i.into();
        trace!("Getting CSR 0x{:x}", i);
        return Ok(if i == MHARTID {
            0
        } else if i == MSTATUS {
            self.mstatus
        } else if i == MEDELEG {
            self.medeleg
        } else if i == MIDELEG {
            self.mideleg
        } else if i == SATP {
            self.satp
        } else if i == MTVEC {
            self.mtvec
        } else if i == MEPC {
            self.mepc
        } else if i == MTVAL {
            self.mtval
        } else {
            error!("unimplemented Csrs.get 0x{:x}", i);
            return Err(());
        });
    }
}

impl fmt::Debug for Csrs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Csrs")
    }
}
