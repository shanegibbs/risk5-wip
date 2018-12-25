pub struct Mstatus(u64);

impl Mstatus {

    pub fn new() -> Self {
        Mstatus(0)
    }

    #[inline(always)]
    pub fn val(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    fn bool_field(&self, offset: u8) -> bool {
        (self.0 >> offset) & 0x1 == 1
    }

    #[inline(always)]
    fn field(&self, offset: u8, len: u8) -> u64 {
        let mask = (2 as u64).pow(len as u32) - 1;
        (self.0 >> offset) & mask
    }

    #[inline(always)]
    fn set_bool_field(&mut self, offset: u8, value: u64) {
        self.0 |= (value & 0x1) << offset
    }

    // mie

    #[inline(always)]
    pub fn machine_interrupt_enabled(&self) -> u64 {
        self.field(3, 1)
    }

    #[inline(always)]
    pub fn set_machine_interrupt_enabled(&mut self, n: u64) {
       self.set_bool_field(3, n)
    }

    // sie

    #[inline(always)]
    pub fn supervisor_interrupt_enabled(&self) -> bool {
       self.bool_field(1)
    }

    #[inline(always)]
    pub fn set_supervisor_interrupt_enabled(&mut self, n: u64) {
       self.set_bool_field(1, n)
    }

    // uie

    #[inline(always)]
    pub fn user_interrupt_enabled(&self) -> bool {
       self.bool_field(0)
    }

    #[inline(always)]
    pub fn set_user_interrupt_enabled(&mut self, n: u64) {
       self.set_bool_field(0, n)
    }

    // mpie

    #[inline(always)]
    pub fn machine_prior_interrupt_enabled(&self) -> bool {
       self.bool_field(7)
    }

    #[inline(always)]
    pub fn set_machine_prior_interrupt_enabled(&mut self, n: u64) {
       self.set_bool_field(7, n)
    }

    #[inline(always)]
    pub fn move_machine_interrupt_enabled_to_prior(&mut self) {
        let mie = self.machine_interrupt_enabled();
        self.set_machine_prior_interrupt_enabled(mie);
    }

    // spie

    #[inline(always)]
    pub fn supervisor_prior_interrupt_enabled(&self) -> bool {
       self.bool_field(5)
    }

    #[inline(always)]
    pub fn user_prior_interrupt_enabled(&self) -> bool {
       self.bool_field(4)
    }

    #[inline(always)]
    pub fn supervisor_xlen(&self) -> u8 {
        (self.0 >> 34 & 0x3) as u8
    }

    #[inline(always)]
    pub fn set_supervisor_xlen(&mut self, n: u64) {
        self.0 |= (n & 0x3) << 34;
    }

    #[inline(always)]
    pub fn user_xlen(&self) -> u8 {
        (self.0 >> 32 & 0x3) as u8
    }

    #[inline(always)]
    pub fn set_user_xlen(&mut self, n: u64) {
        self.0 |= (n & 0x3) << 32;
    }

}

use std::fmt;
impl fmt::Debug for Mstatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mstatus mie={} sie={} uie={} mpie={} spie={} upie={} sxl={} uxl={}",
               self.machine_interrupt_enabled(),
               self.supervisor_interrupt_enabled(),
               self.user_interrupt_enabled(),
               self.machine_prior_interrupt_enabled(),
               self.supervisor_prior_interrupt_enabled(),
               self.user_prior_interrupt_enabled(),
               self.supervisor_xlen(),
               self.user_xlen(),
               )
    }
}
