use std::collections::HashMap;

pub struct Memory {
    data: HashMap<usize, u8>,
    blocks: Vec<(usize, Vec<u8>)>,
}

impl Memory {
    pub fn new(_mb: usize) -> Self {
        // let mem_size = mb << 20; // 15 MB
        // let mem = vec![0; mem_size];
        Memory {
            data: HashMap::new(),
            blocks: vec![],
        }
    }

    pub fn add_block(&mut self, offset: usize, size: usize) {
        trace!("Adding memory block at 0x{:x} of size 0x{:x} as index {}",
               offset,
               size,
               self.blocks.len());
        let mem = vec![0; size];
        self.blocks.push((offset, mem));
    }

    fn find_block_for(&self, offset: usize) -> usize {
        let mut c = self.blocks.len();
        while c != 0 {
            let i = c - 1;
            if offset >= self.blocks[i].0 {
                if offset > self.blocks[i].0 + self.blocks[i].1.len() {
                    panic!("Memory out of range. Unable to read 0x{:x}", offset);
                }
                return i;
            }
            c -= 1;
        }
        unreachable!()
    }

    pub fn write_b(&mut self, offset: usize, value: u8) {
        let block = self.find_block_for(offset);
        let block_offset = offset - self.blocks[block].0;
        self.blocks[block].1[block_offset] = value;
    }

    pub fn read_b(&self, offset: usize) -> u8 {
        let block = self.find_block_for(offset);
        let block_offset = offset - self.blocks[block].0;
        return self.blocks[block].1[block_offset];
    }

    pub fn read_d(&self, offset: u64) -> u64 {
        0
    }

    pub fn read_w(&self, offset: u64) -> u32 {
        trace!("Reading word at 0x{:x}", offset);

        let offset = offset as usize;
        let block_i = self.find_block_for(offset);
        let block = &self.blocks[block_i];

        let p1 = block.1[offset - block.0] as u32;
        let p2 = block.1[offset - block.0 + 1] as u32;
        let p3 = block.1[offset - block.0 + 2] as u32;
        let p4 = block.1[offset - block.0 + 3] as u32;

        let mut v = p1;
        v = v | (p2 << 8);
        v = v | (p3 << 16);
        v = v | (p4 << 24);
        trace!("read 0x{:x}", v);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_env_logger;

    #[test]
    fn it_memory() {
        let _ = pretty_env_logger::try_init();
        let mut m = Memory::new(15);
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
