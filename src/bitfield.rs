mod mstatus;
mod satp;

pub(crate) use self::mstatus::Mstatus;
pub(crate) use self::satp::Satp;

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
    fn _bool_field(&self, offset: u8) -> bool {
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
