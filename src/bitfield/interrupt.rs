use crate::bitfield::BitField;

pub(crate) struct Interrupt(BitField);

impl Interrupt {
    #[inline(always)]
    pub fn val(&self) -> u64 {
        self.0.val()
    }

    // mei

    #[inline(always)]
    pub fn machine_external_interrupt(&self) -> u64 {
        self.0.field(11, 1)
    }

    #[inline(always)]
    pub fn set_machine_external_interrupt(&mut self, n: u64) {
        self.0.set_bool_field(11, n)
    }

    // sei

    #[inline(always)]
    pub fn supervisor_external_interrupt(&self) -> u64 {
        self.0.field(9, 1)
    }

    #[inline(always)]
    pub fn set_supervisor_external_interrupt(&mut self, n: u64) {
        self.0.set_bool_field(9, n)
    }

    // uei

    #[inline(always)]
    pub fn user_external_interrupt(&self) -> u64 {
        self.0.field(8, 1)
    }

    #[inline(always)]
    pub fn set_user_external_interrupt(&mut self, n: u64) {
        self.0.set_bool_field(8, n)
    }

    // mti

    #[inline(always)]
    pub fn machine_timer_interrupt(&self) -> u64 {
        self.0.field(7, 1)
    }

    #[inline(always)]
    pub fn set_machine_timer_interrupt(&mut self, n: u64) {
        self.0.set_bool_field(7, n)
    }

    // sti

    #[inline(always)]
    pub fn supervisor_timer_interrupt(&self) -> u64 {
        self.0.field(5, 1)
    }

    #[inline(always)]
    pub fn set_supervisor_timer_interrupt(&mut self, n: u64) {
        self.0.set_bool_field(5, n)
    }

    // uti

    #[inline(always)]
    pub fn user_timer_interrupt(&self) -> u64 {
        self.0.field(4, 1)
    }

    #[inline(always)]
    pub fn set_user_timer_interrupt(&mut self, n: u64) {
        self.0.set_bool_field(4, n)
    }

    // msi

    #[inline(always)]
    pub fn machine_software_interrupt(&self) -> u64 {
        self.0.field(3, 1)
    }

    #[inline(always)]
    pub fn set_machine_software_interrupt(&mut self, n: u64) {
        self.0.set_bool_field(3, n)
    }

    // ssi

    #[inline(always)]
    pub fn supervisor_software_interrupt(&self) -> u64 {
        self.0.field(1, 1)
    }

    #[inline(always)]
    pub fn set_supervisor_software_interrupt(&mut self, n: u64) {
        self.0.set_bool_field(1, n)
    }

    // usi

    #[inline(always)]
    pub fn user_software_interrupt(&self) -> u64 {
        self.0.field(0, 1)
    }

    #[inline(always)]
    pub fn set_user_software_interrupt(&mut self, n: u64) {
        self.0.set_bool_field(0, n)
    }

    #[inline(always)]
    pub fn supervisor_view(&self) -> u64 {
        self.0.val() & SMASK_READ
    }

    #[inline(always)]
    pub fn set_supervisor_vals(&mut self, v: u64) {
        (self.0).0 &= !SMASK_WRITE;
        (self.0).0 |= v & SMASK_WRITE;
    }
}

const _UMASK_WRITE: u64 = 1 + (1 << 4) + (1 << 8);
const SMASK_WRITE: u64 = (1 << 1) + (1 << 5) + (1 << 9);

const UMASK_READ: u64 = 1 + (1 << 4) + (1 << 8);
const SMASK_READ: u64 = (1 << 1) + (1 << 5) + (1 << 9) + UMASK_READ;

impl Default for Interrupt {
    fn default() -> Self {
        Interrupt(BitField::new(0))
    }
}

impl From<Interrupt> for u64 {
    fn from(i: Interrupt) -> u64 {
        i.val()
    }
}

impl From<&Interrupt> for u64 {
    fn from(i: &Interrupt) -> u64 {
        i.val()
    }
}

impl From<u64> for Interrupt {
    fn from(i: u64) -> Interrupt {
        Interrupt(BitField::new(i))
    }
}

use std::fmt;
impl fmt::Debug for Interrupt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "interrupt mei={}, sei={} uei={} mti={} sti={} uti={} msi={} ssi={} usi={}",
            self.machine_external_interrupt(),
            self.supervisor_external_interrupt(),
            self.user_external_interrupt(),
            self.machine_timer_interrupt(),
            self.supervisor_timer_interrupt(),
            self.user_timer_interrupt(),
            self.machine_software_interrupt(),
            self.supervisor_software_interrupt(),
            self.user_software_interrupt(),
        )
    }
}
