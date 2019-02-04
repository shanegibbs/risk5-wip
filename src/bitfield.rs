mod interrupt;
mod mstatus;
mod satp;

pub(crate) use self::interrupt::Interrupt;
pub(crate) use self::mstatus::Mstatus;
pub(crate) use self::satp::Satp;

pub(crate) struct PhysicalAddress(BitField);
impl PhysicalAddress {
    pub fn offset(&self) -> u64 {
        self.0.field(0, 12)
    }
    pub fn set_offset(&mut self, offset: u64) {
        self.0.set_field(0, 12, offset)
    }

    pub fn set_physical_page_number_arr(&mut self, i: u8, val: u64) {
        match i {
            0 | 1 => self.0.set_field(12 + (i * 9), 9, val),
            2 => self.0.set_field(30, 26, val),
            _ => unreachable!(),
        }
    }

    pub fn val(&self) -> u64 {
        self.0.val()
    }
}

impl Into<PhysicalAddress> for u64 {
    fn into(self) -> PhysicalAddress {
        PhysicalAddress(BitField::new(self))
    }
}

impl Into<u64> for PhysicalAddress {
    fn into(self) -> u64 {
        self.0.into_val()
    }
}

pub(crate) struct VirtualAddress(BitField);
impl VirtualAddress {
    pub fn virtual_page_number(&self, i: u8) -> u64 {
        assert!(i < 3);
        self.0.field(12 + (i * 9), 9)
    }
    pub fn offset(&self) -> u64 {
        self.0.field(0, 12)
    }
    pub fn val(&self) -> u64 {
        self.0.val()
    }
}

impl Into<VirtualAddress> for u64 {
    fn into(self) -> VirtualAddress {
        VirtualAddress(BitField::new(self))
    }
}

pub(crate) struct PageTableEntry(BitField);
impl PageTableEntry {
    pub fn v(&self) -> bool {
        self.0.bool_field(0)
    }

    pub fn r(&self) -> bool {
        self.0.bool_field(1)
    }

    pub fn w(&self) -> bool {
        self.0.bool_field(2)
    }

    pub fn x(&self) -> bool {
        self.0.bool_field(3)
    }

    pub fn physical_page_number_arr(&self, i: u8) -> u64 {
        match i {
            2 => self.0.field(28, 26),
            0 | 1 => self.0.field(10 + (i * 9), 9),
            _ => unreachable!(),
        }
    }

    pub fn physical_page_number(&self) -> u64 {
        self.0.field(10, 44)
    }
    pub fn offset(&self) -> u64 {
        self.0.field(0, 12)
    }

    pub fn val(&self) -> u64 {
        self.0.val()
    }
}

impl Into<PageTableEntry> for u64 {
    fn into(self) -> PageTableEntry {
        PageTableEntry(BitField::new(self))
    }
}

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

    #[inline(always)]
    pub fn into_val(self) -> u64 {
        self.0
    }

    #[inline(always)]
    fn field(&self, offset: u8, len: u8) -> u64 {
        let mask = (2 as u64).pow(len as u32) - 1;
        (self.0 >> offset) & mask
    }

    #[inline(always)]
    fn set_field(&mut self, offset: u8, len: u8, val: u64) {
        // valid bits
        let mask = (1 << len) - 1;
        // make sure val only includes valid bits
        let val = val & mask;
        // move mask to offset
        let mask = mask << offset;
        // inverse mask. valid bits are zero
        let neg_mask = !mask;

        // set field to zeros
        self.0 &= neg_mask;
        // set ones
        self.0 |= val << offset;
    }

    #[inline(always)]
    fn val(&self) -> u64 {
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

        // assert_eq!(BitField::new(1).with_field(0, 1, 0).into_val(), 0);

        assert_eq!(BitField::new(0).with_field(0, 1, 1).field(0, 1), 1);
        assert_eq!(BitField::new(0).with_field(0, 1, 3).field(0, 1), 1);
        assert_eq!(BitField::new(0).with_field(1, 1, 1).field(1, 1), 1);
        assert_eq!(BitField::new(0).with_field(2, 1, 1).field(2, 1), 1);

        assert_eq!(
            BitField::new(3)
                .with_field(0, 1, 1)
                .with_field(1, 1, 1)
                .into_val(),
            3
        );
        assert_eq!(BitField::new(3).with_field(0, 2, 3).into_val(), 3);

        assert_eq!(
            BitField::new(0)
                .with_field(0, 1, 1)
                .with_field(1, 1, 1)
                .field(0, 2),
            3
        );

        assert_eq!(
            BitField::new(0)
                .with_field(0, 1, 1)
                .with_field(2, 1, 1)
                .field(0, 3),
            5
        );
    }
}
