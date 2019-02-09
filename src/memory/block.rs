use super::Memory;
use std::fmt;

pub struct BlockMemory {
    blocks: Vec<(u64, Vec<u8>)>,
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
        self.blocks.push((offset, mem));
    }

    fn find_block_for(&self, offset: u64) -> u64 {
        let mut c = self.blocks.len();
        while c != 0 {
            let i = c - 1;
            if offset >= self.blocks[i].0
                && offset < self.blocks[i].0 + (self.blocks[i].1.len() as u64)
            {
                /*
                if offset > self.blocks[i].0 + self.blocks[i].1.len() as u64 {
                    panic!(
                        "Memory out of range. Unable to find block for 0x{:x}",
                        offset
                    );
                }
                */
                return i as u64;
            }
            c -= 1;
        }
        error!("Unable to find memory block for address 0x{:x}", offset);
        panic!("Unable to find memory block");
    }
}

impl Memory for BlockMemory {
    fn read_b(&mut self, offset: u64) -> u8 {
        let block_i = self.find_block_for(offset);
        let block = &self.blocks[block_i as usize];
        let offset = (offset - block.0) as usize;
        block.1[offset]
    }

    fn read_h(&mut self, offset: u64) -> u16 {
        let block_i = self.find_block_for(offset);
        let block = &self.blocks[block_i as usize];
        let offset = (offset - block.0) as usize;

        let mut v: u16 = 0;
        unsafe {
            core::ptr::copy_nonoverlapping(
                (&block.1[offset..]).as_ptr(),
                &mut v as *mut u16 as *mut u8,
                2,
            );
        }
        v
    }

    fn read_w(&mut self, offset: u64) -> u32 {
        let block_i = self.find_block_for(offset);
        let block = &self.blocks[block_i as usize];
        let offset = (offset - block.0) as usize;

        let mut v: u32 = 0;
        unsafe {
            core::ptr::copy_nonoverlapping(
                (&block.1[offset..]).as_ptr(),
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

    fn read_d(&mut self, offset: u64) -> u64 {
        let block_i = self.find_block_for(offset);
        let block = &self.blocks[block_i as usize];
        let offset = (offset - block.0) as usize;

        let mut v: u64 = 0;
        unsafe {
            core::ptr::copy_nonoverlapping(
                (&block.1[offset..]).as_ptr(),
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
        let block = self.find_block_for(offset) as usize;
        let offset = (offset - self.blocks[block].0) as usize;
        self.blocks[block].1[offset] = value;
    }

    fn write_h(&mut self, offset: u64, value: u16) {
        let block_i = self.find_block_for(offset) as usize;
        let block = &mut self.blocks[block_i];
        let offset = (offset - block.0) as usize;

        unsafe {
            let bytes = *(&value as *const u16 as *const [u8; 2]);
            core::ptr::copy_nonoverlapping(
                (&bytes).as_ptr(),
                (&mut block.1[offset..]).as_mut_ptr(),
                2,
            );
        }
    }

    fn write_w(&mut self, offset: u64, value: u32) {
        let block_i = self.find_block_for(offset) as usize;
        let block = &mut self.blocks[block_i];
        let offset = (offset - block.0) as usize;

        unsafe {
            let bytes = *(&value as *const u32 as *const [u8; 4]);
            core::ptr::copy_nonoverlapping(
                (&bytes).as_ptr(),
                (&mut block.1[offset..]).as_mut_ptr(),
                4,
            );
        }

        // self.blocks[block].1[offset] = value as u8;
        // self.blocks[block].1[offset + 1] = (value >> 8) as u8;
        // self.blocks[block].1[offset + 2] = (value >> 16) as u8;
        // self.blocks[block].1[offset + 3] = (value >> 24) as u8;
    }

    fn write_d(&mut self, offset: u64, value: u64) {
        let block_i = self.find_block_for(offset) as usize;
        let block = &mut self.blocks[block_i];
        let offset = (offset - block.0) as usize;

        unsafe {
            let bytes = *(&value as *const u64 as *const [u8; 8]);
            core::ptr::copy_nonoverlapping(
                (&bytes).as_ptr(),
                (&mut block.1[offset..]).as_mut_ptr(),
                8,
            );
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
