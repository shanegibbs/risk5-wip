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
    fn write_b(&mut self, offset: u64, value: u8) {
        let block = self.find_block_for(offset) as usize;
        let block_offset = offset - self.blocks[block].0;
        self.blocks[block].1[block_offset as usize] = value;
    }

    fn read_b(&self, offset: u64) -> u8 {
        let block = self.find_block_for(offset) as usize;
        let block_offset = offset - self.blocks[block].0;
        return self.blocks[block].1[block_offset as usize];
    }

    /*
    fn read_w(&self, offset: u64) -> u32 {
        trace!("Reading word at 0x{:x}", offset);

        let block_i = self.find_block_for(offset);
        let block = &self.blocks[block_i as usize];

        let p1 = block.1[(offset - block.0) as usize] as u32;
        let p2 = block.1[(offset - block.0 + 1) as usize] as u32;
        let p3 = block.1[(offset - block.0 + 2) as usize] as u32;
        let p4 = block.1[(offset - block.0 + 3) as usize] as u32;

        let mut v = p1;
        v = v | (p2 << 8);
        v = v | (p3 << 16);
        v = v | (p4 << 24);
        trace!("read 0x{:x}", v);
        v
    }
    */
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
