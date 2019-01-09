use crate::bitfield::BitField;

pub(crate) struct Satp(BitField);

impl Satp {
    #[inline(always)]
    pub fn ppn(&self) -> u64 {
        self.0.field(0, 44)
    }

    #[inline(always)]
    pub fn asid(&self) -> u64 {
        self.0.field(44, 16)
    }

    #[inline(always)]
    pub fn mode(&self) -> u64 {
        self.0.field(60, 4)
    }
}

impl Default for Satp {
    fn default() -> Self {
        0.into()
    }
}

impl Into<Satp> for u64 {
    fn into(self) -> Satp {
        Satp(BitField::new(self))
    }
}

impl Into<u64> for &Satp {
    fn into(self) -> u64 {
        self.0.val()
    }
}
