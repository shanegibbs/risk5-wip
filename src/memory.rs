mod block;
mod bytemap;
mod fake;
// mod sv39;

pub(crate) use self::block::BlockMemory;
pub(crate) use self::bytemap::ByteMap;
#[cfg(test)]
pub(crate) use self::fake::*;

pub trait Memory {
    fn read_b(&mut self, offset: u64) -> u8;
    fn write_b(&mut self, offset: u64, value: u8);

    fn write_h(&mut self, offset: u64, value: u16) {
        self.write_b(offset, value as u8);
        self.write_b(offset + 1, (value >> 8) as u8);
    }

    fn write_w(&mut self, offset: u64, value: u32) {
        self.write_b(offset, value as u8);
        self.write_b(offset + 1, (value >> 8) as u8);
        self.write_b(offset + 2, (value >> 16) as u8);
        self.write_b(offset + 3, (value >> 24) as u8);
    }

    fn write_d(&mut self, offset: u64, value: u64) {
        self.write_b(offset, value as u8);
        self.write_b(offset + 1, (value >> 8) as u8);
        self.write_b(offset + 2, (value >> 16) as u8);
        self.write_b(offset + 3, (value >> 24) as u8);
        self.write_b(offset + 4, (value >> 32) as u8);
        self.write_b(offset + 5, (value >> 40) as u8);
        self.write_b(offset + 6, (value >> 48) as u8);
        self.write_b(offset + 7, (value >> 56) as u8);
    }

    fn read_h(&mut self, offset: u64) -> u16 {
        let mut n = self.read_b(offset) as u16;
        n |= (self.read_b(offset + 1) as u16) << 8;
        n
    }

    fn read_w(&mut self, offset: u64) -> u32 {
        let mut n = self.read_b(offset) as u32;
        n |= (self.read_b(offset + 1) as u32) << 8;
        n |= (self.read_b(offset + 2) as u32) << 16;
        n |= (self.read_b(offset + 3) as u32) << 24;
        n
    }

    fn read_d(&mut self, offset: u64) -> u64 {
        let mut n = self.read_b(offset) as u64;
        n |= (self.read_b(offset + 1) as u64) << 8;
        n |= (self.read_b(offset + 2) as u64) << 16;
        n |= (self.read_b(offset + 3) as u64) << 24;
        n |= (self.read_b(offset + 4) as u64) << 32;
        n |= (self.read_b(offset + 5) as u64) << 40;
        n |= (self.read_b(offset + 6) as u64) << 48;
        n |= (self.read_b(offset + 7) as u64) << 56;
        n
    }
}
