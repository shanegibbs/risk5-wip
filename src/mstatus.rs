struct BitField(u64);

impl BitField {
    #[inline(always)]
    fn new(i: u64) -> Self {
        BitField(i)
    }

    #[cfg(test)]
    fn with_field(mut self, offset: u8, len: u8, val: u64) -> Self {
        self.set_field(offset, len, val);
        self
    }
    #[cfg(test)]
    fn into_val(self) -> u64 {
        self.0
    }

    #[inline(always)]
    fn field(&self, offset: u8, len: u8) -> u64 {
        let mask = (2 as u64).pow(len as u32) - 1;
        (self.0 >> offset) & mask
    }
    #[inline(always)]
    fn set_field(&mut self, offset: u8, len: u8, val: u64) {
        let mask = (2 as u64).pow(len as u32) - 1;
        self.0 |= (val & mask) << offset;
    }
    #[inline(always)]
    pub fn val(&self) -> u64 {
        self.0
    }
    #[inline(always)]
    fn bool_field(&self, offset: u8) -> bool {
        self.field(offset, 1) == 1
    }
    #[inline(always)]
    fn set_bool_field(&mut self, offset: u8, value: u64) {
        self.set_field(offset, 1, value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bit_field() {
        assert_eq!(BitField::new(0).into_val(), 0);
        assert_eq!(BitField::new(0).with_field(0, 1, 1).into_val(), 1);
        assert_eq!(BitField::new(0).with_field(0, 1, 3).into_val(), 1);
        assert_eq!(BitField::new(0).with_field(1, 1, 1).into_val(), 2);
        assert_eq!(BitField::new(0).with_field(2, 1, 1).into_val(), 4);

        assert_eq!(BitField::new(0).with_field(0, 1, 1).field(0, 1), 1);
        assert_eq!(BitField::new(0).with_field(0, 1, 3).field(0, 1), 1);
        assert_eq!(BitField::new(0).with_field(1, 1, 1).field(1, 1), 1);
        assert_eq!(BitField::new(0).with_field(2, 1, 1).field(2, 1), 1);

        assert_eq!(BitField::new(3).with_field(0, 1, 1).with_field(1, 1, 1).into_val(), 3);
        assert_eq!(BitField::new(3).with_field(0, 2, 3).into_val(), 3);

        assert_eq!(BitField::new(0)
                   .with_field(0, 1, 1)
                   .with_field(1, 1, 1)
                   .field(0, 2), 3);

        assert_eq!(BitField::new(0)
                   .with_field(0, 1, 1)
                   .with_field(2, 1, 1)
                   .field(0, 3), 5);
    }
}

pub struct Mstatus(BitField);

impl Mstatus {

    pub fn new() -> Self {
        Mstatus(BitField::new(0))
    }

    pub fn new_with_val(i: u64) -> Self {
        Mstatus(BitField::new(i))
    }

    #[inline(always)]
    pub fn val(&self) -> u64 {
        self.0.val()
    }

    // mie

    #[inline(always)]
    pub fn machine_interrupt_enabled(&self) -> u64 {
        self.0.field(3, 1)
    }

    #[inline(always)]
    pub fn set_machine_interrupt_enabled(&mut self, n: u64) {
       self.0.set_bool_field(3, n)
    }

    // sie

    #[inline(always)]
    pub fn supervisor_interrupt_enabled(&self) -> u64 {
       self.0.field(1, 1)
    }

    #[inline(always)]
    pub fn set_supervisor_interrupt_enabled(&mut self, n: u64) {
       self.0.set_bool_field(1, n)
    }

    // uie

    #[inline(always)]
    pub fn user_interrupt_enabled(&self) -> u64 {
       self.0.field(0, 1)
    }

    #[inline(always)]
    pub fn set_user_interrupt_enabled(&mut self, n: u64) {
       self.0.set_bool_field(0, n)
    }

    // mpie

    #[inline(always)]
    pub fn machine_prior_interrupt_enabled(&self) -> u64 {
       self.0.field(7, 1)
    }

    #[inline(always)]
    pub fn set_machine_prior_interrupt_enabled(&mut self, n: u64) {
       self.0.set_bool_field(7, n)
    }

    #[inline(always)]
    pub fn move_machine_interrupt_enabled_to_prior(&mut self) {
        let mie = self.machine_interrupt_enabled();
        self.set_machine_prior_interrupt_enabled(mie);
    }

    // spie

    #[inline(always)]
    pub fn supervisor_prior_interrupt_enabled(&self) -> u64 {
       self.0.field(5, 1)
    }

    #[inline(always)]
    pub fn user_prior_interrupt_enabled(&self) -> u64 {
       self.0.field(4, 1)
    }

    // sxl

    #[inline(always)]
    pub fn supervisor_xlen(&self) -> u64 {
        self.0.field(34, 2)
    }

    #[inline(always)]
    pub fn set_supervisor_xlen(&mut self, n: u64) {
        self.0.set_field(34, 2, n)
    }

    // uxl

    #[inline(always)]
    pub fn user_xlen(&self) -> u64 {
        self.0.field(32, 2)
    }

    #[inline(always)]
    pub fn set_user_xlen(&mut self, n: u64) {
        self.0.set_field(32, 2, n)
    }

    // mpp

    #[inline(always)]
    pub fn machine_previous_privilege(&self) -> u64 {
        self.0.field(11, 2)
    }

    #[inline(always)]
    pub fn set_machine_previous_privilege(&mut self, n: u64) {
        self.0.set_field(11, 2, n)
    }

    // spp

    #[inline(always)]
    pub fn supervisor_previous_privilege(&self) -> u64 {
        self.0.field(8, 2)
    }

    #[inline(always)]
    pub fn set_supervisor_previous_privilege(&mut self, n: u64) {
        self.0.set_field(8, 2, n)
    }

    // fs

    #[inline(always)]
    pub fn floating_point_state(&self) -> u64 {
        self.0.field(13, 2)
    }

    // xs

    #[inline(always)]
    pub fn extensions_state(&self) -> u64 {
        self.0.field(15, 2)
    }

}

use std::fmt;
impl fmt::Debug for Mstatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mstatus mie={} sie={} uie={} mpie={} spie={} upie={} mpp={} spp={} fs={} xs={} sxl={} uxl={}",
               self.machine_interrupt_enabled(),
               self.supervisor_interrupt_enabled(),
               self.user_interrupt_enabled(),
               self.machine_prior_interrupt_enabled(),
               self.supervisor_prior_interrupt_enabled(),
               self.user_prior_interrupt_enabled(),
               self.machine_previous_privilege(),
               self.supervisor_previous_privilege(),
               self.floating_point_state(),
               self.extensions_state(),
               self.supervisor_xlen(),
               self.user_xlen(),
               )
    }
}
