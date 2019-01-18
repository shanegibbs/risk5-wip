use crate::bitfield::BitField;

pub(crate) struct Mstatus(BitField);

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
    pub fn _set_user_interrupt_enabled(&mut self, n: u64) {
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
    pub fn set_supervisor_prior_interrupt_enabled(&mut self, ie: u64) {
        self.0.set_field(5, 1, ie)
    }

    #[inline(always)]
    pub fn move_supervisor_interrupt_enabled_to_prior(&mut self) {
        let ie = self.supervisor_interrupt_enabled();
        self.set_supervisor_prior_interrupt_enabled(ie);
    }

    // upie

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
        self.0.field(8, 1)
    }

    #[inline(always)]
    pub fn set_supervisor_previous_privilege(&mut self, n: u64) {
        self.0.set_field(8, 1, n)
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
