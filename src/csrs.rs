pub struct Csrs {
    regs: [u64; 4096],
}

impl Csrs {
    pub fn new() -> Self {
        Csrs { regs: [0; 4096] }
    }
    pub fn set<T: Into<u64>>(&mut self, i: T, v: u64) {
        let i = i.into();
        debug!("Setting CSR 0x{:x} to 0x{:x}", i, v);
        self.regs[i as usize] = v;
    }
    pub fn get<T: Into<u64>>(&self, i: T) -> u64 {
        let i = i.into();
        trace!("Getting CSR 0x{:x}", i);
        self.regs[i as usize]
    }
}
