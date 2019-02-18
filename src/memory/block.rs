use super::Memory;
use core::ptr::copy_nonoverlapping;
use std::fmt;

#[derive(Clone)]
struct Block {
    start: u64,
    end: u64,
    size: u64,
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct BlockMemory {
    blocks: Vec<Block>,
}

impl fmt::Debug for BlockMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockMemory")
    }
}

impl BlockMemory {
    pub fn new(_mb: u64) -> Self {
        // let mem_size = mb << 20; // 15 MB
        // let mem = vec![0; mem_size];
        Self { blocks: vec![] }
    }

    pub fn add_block(&mut self, offset: u64, size: u64) {
        debug!(
            "Adding memory block at 0x{:x} (to 0x{:x}) of size 0x{:x} as index {}",
            offset,
            offset + size,
            size,
            self.blocks.len()
        );
        let mem = vec![0; size as usize];
        self.blocks.push(Block {
            start: offset,
            end: offset + size,
            size,
            data: mem,
        });
    }

    fn get_block(&self, i: usize) -> &Block {
        unsafe { self.blocks.get_unchecked(i) }
    }

    fn get_block_mut(&mut self, i: usize) -> &mut Block {
        unsafe { self.blocks.get_unchecked_mut(i) }
    }

    #[inline(never)]
    fn find_block_for(&self, offset: u64) -> usize {
        if offset >= 0x2000000 && offset < 0x20c0000 {
            info!("clint 0x{:x}", offset);
        }
        // if offset >= 0x50000000 && offset < 0x50000100 {
        //     error!("serial 0x{:x}", offset);
        // }
        for (i, block) in self.blocks.iter().enumerate() {
            if offset >= block.start {
                assert!(offset < block.end);
                return i;
            }
        }
        panic!("Unable to find memory block");
    }
}

impl Memory for BlockMemory {
    fn read_b(&mut self, offset: u64) -> u8 {
        let block = self.get_block(self.find_block_for(offset));
        let offset = (offset - block.start) as usize;
        unsafe { *block.data.get_unchecked(offset) }
    }

    fn read_h(&mut self, offset: u64) -> u16 {
        let block = self.get_block(self.find_block_for(offset));
        let offset = (offset - block.start) as usize;

        let mut v: u16 = 0;
        unsafe {
            copy_nonoverlapping(
                block.data.as_ptr().add(offset),
                &mut v as *mut u16 as *mut u8,
                2,
            );
        }
        v
    }

    fn read_w(&mut self, offset: u64) -> u32 {
        let block = self.get_block(self.find_block_for(offset));
        let offset = (offset - block.start) as usize;

        let mut v: u32 = 0;
        unsafe {
            copy_nonoverlapping(
                block.data.as_ptr().add(offset),
                &mut v as *mut u32 as *mut u8,
                4,
            );
        }
        v

        // block.1[(offset) as usize] as u32
        //     + ((block.1[(offset + 1) as usize] as u32) << 8)
        //     + ((block.1[(offset + 2) as usize] as u32) << 16)
        //     + ((block.1[(offset + 3) as usize] as u32) << 24)
    }

    #[inline(never)]
    fn read_d(&mut self, offset: u64) -> u64 {
        let block = self.get_block(self.find_block_for(offset));
        let offset = (offset - block.start) as usize;

        let mut v: u64 = 0;
        unsafe {
            copy_nonoverlapping(
                block.data.as_ptr().add(offset),
                &mut v as *mut u64 as *mut u8,
                8,
            );
        }
        v

        // block.1[(offset) as usize] as u64
        //     + ((block.1[(offset + 1) as usize] as u64) << 8)
        //     + ((block.1[(offset + 2) as usize] as u64) << 16)
        //     + ((block.1[(offset + 3) as usize] as u64) << 24)
        //     + ((block.1[(offset + 4) as usize] as u64) << 32)
        //     + ((block.1[(offset + 5) as usize] as u64) << 40)
        //     + ((block.1[(offset + 6) as usize] as u64) << 48)
        //     + ((block.1[(offset + 7) as usize] as u64) << 56)
    }

    fn write_b(&mut self, offset: u64, value: u8) {
        let block = self.get_block_mut(self.find_block_for(offset));
        let offset = (offset - block.start) as usize;
        unsafe { *block.data.get_unchecked_mut(offset) = value }
    }

    fn write_h(&mut self, offset: u64, value: u16) {
        let block = self.get_block_mut(self.find_block_for(offset));
        let offset = (offset - block.start) as usize;
        unsafe {
            let bytes = *(&value as *const u16 as *const [u8; 2]);
            copy_nonoverlapping(bytes.as_ptr(), block.data.as_mut_ptr().add(offset), 2);
        }
    }

    fn write_w(&mut self, offset: u64, value: u32) {
        let block = self.get_block_mut(self.find_block_for(offset));
        let offset = (offset - block.start) as usize;
        unsafe {
            let bytes = *(&value as *const u32 as *const [u8; 4]);
            copy_nonoverlapping(bytes.as_ptr(), block.data.as_mut_ptr().add(offset), 4);
        }

        // self.blocks[block].1[offset] = value as u8;
        // self.blocks[block].1[offset + 1] = (value >> 8) as u8;
        // self.blocks[block].1[offset + 2] = (value >> 16) as u8;
        // self.blocks[block].1[offset + 3] = (value >> 24) as u8;
    }

    fn write_d(&mut self, offset: u64, value: u64) {
        let block = self.get_block_mut(self.find_block_for(offset));
        let offset = (offset - block.start) as usize;
        unsafe {
            let bytes = *(&value as *const u64 as *const [u8; 8]);
            copy_nonoverlapping(bytes.as_ptr(), block.data.as_mut_ptr().add(offset), 8);
        }

        // self.blocks[block].1[offset] = value as u8;
        // self.blocks[block].1[offset + 1] = (value >> 8) as u8;
        // self.blocks[block].1[offset + 2] = (value >> 16) as u8;
        // self.blocks[block].1[offset + 3] = (value >> 24) as u8;
        // self.blocks[block].1[offset + 4] = (value >> 32) as u8;
        // self.blocks[block].1[offset + 5] = (value >> 40) as u8;
        // self.blocks[block].1[offset + 6] = (value >> 48) as u8;
        // self.blocks[block].1[offset + 7] = (value >> 56) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_env_logger;

    #[test]
    fn rw() {
        let _ = pretty_env_logger::try_init();
        let mut m = BlockMemory::new(0);
        m.add_block(0, 10);

        m.write_b(0x0, 0x01);
        m.write_b(0x1, 0x02);
        m.write_b(0x2, 0x03);
        m.write_b(0x3, 0x04);
        m.write_b(0x4, 0x05);
        m.write_b(0x5, 0x06);
        m.write_b(0x6, 0x07);
        m.write_b(0x7, 0x08);
        assert_eq!(m.read_b(0x0), 0x01);
        assert_eq!(m.read_h(0x0), 0x0201);
        assert_eq!(m.read_w(0x0), 0x04030201);
        assert_eq!(m.read_d(0x0), 0x0807060504030201);
    }

    #[test]
    fn it_memory() {
        let _ = pretty_env_logger::try_init();
        let mut m = BlockMemory::new(15);
        m.add_block(0x10, 0x5);
        m.add_block(0x20, 0x6);

        // normal bounds
        m.write_b(0x12, 0x1);
        m.write_b(0x22, 0x2);

        assert_eq!(m.read_b(0x12), 0x1);
        assert_eq!(m.read_b(0x22), 0x2);

        // upper bounds
        m.write_b(0x14, 0x3);
        m.write_b(0x25, 0x4);

        assert_eq!(m.read_b(0x14), 0x3);
        assert_eq!(m.read_b(0x25), 0x4);

        // lower bounds
        m.write_b(0x10, 0x5);
        m.write_b(0x20, 0x6);

        assert_eq!(m.read_b(0x10), 0x5);
        assert_eq!(m.read_b(0x20), 0x6);
    }
}
