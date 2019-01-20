pub(crate) static REG_NAMES: &'static [&str] = &[
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "sA", "sB", "t3", "t4", "t5",
    "t6",
];

#[derive(Debug)]
pub(crate) struct Regs {
    regs: [u64; 32],
}

impl Regs {
    pub fn new() -> Self {
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
    pub fn geti<T: Into<usize>>(&self, i: T) -> i64 {
        let i = i.into();
        let v = self.regs[i];
        // trace!("Getting reg 0x{:x} 0x{:x}", i, v);
        v as i64
    }

    #[inline(always)]
    pub fn set<T: Into<usize>>(&mut self, i: T, v: u64) {
        let i = i.into();
        // reg 0 is a black hole
        if i == 0 {
            return;
        }
        debug!("Setting reg 0x{:x} 0x{:x}", i, v);
        self.regs[i] = v;
    }

    #[inline(always)]
    pub fn seti<T: Into<usize>>(&mut self, i: T, v: i64) {
        self.set(i, v as u64)
    }
}
