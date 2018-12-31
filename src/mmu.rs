use std::fmt;

pub trait Memory {
    fn write_b(&mut self, _offset: u64, _value: u8) {
        warn!("Memory write_b not implemented")
    }
    fn write_w(&mut self, _offset: u64, _value: u32) {
        warn!("Memory write_w not implemented")
    }
    fn write_d(&mut self, _offset: u64, _value: u64) {
        warn!("Memory write_d not implemented")
    }
    fn read_b(&self, _offset: u64) -> u8 {
        warn!("Memory read_b not implemented");
        unimplemented!()
    }
    fn read_w(&self, _offset: u64) -> u32 {
        warn!("Memory read_w not implemented");
        unimplemented!()
    }
    fn read_d(&self, _offset: u64) -> u64 {
        warn!("Memory read_d not implemented");
        unimplemented!()
    }
}

pub struct BlockMemory {
    blocks: Vec<(u64, Vec<u8>)>,
}

impl fmt::Debug for BlockMemory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        trace!(
            "Adding memory block at 0x{:x} of size 0x{:x} as index {}",
            offset,
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
            if offset >= self.blocks[i].0 {
                if offset > self.blocks[i].0 + self.blocks[i].1.len() as u64 {
                    panic!("Memory out of range. Unable to read 0x{:x}", offset);
                }
                return i as u64;
            }
            c -= 1;
        }
        panic!("Unable to find memory block")
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

    fn read_d(&self, _offset: u64) -> u64 {
        0
    }

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
}

#[derive(Debug)]
enum FakeMemoryItem {
    Byte(u8),
    Word(u32),
    Double(u64),
}

use std::cell::RefCell;
pub struct FakeMemory {
    next: RefCell<Vec<FakeMemoryItem>>,
}

impl fmt::Debug for FakeMemory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FakeMemory")
    }
}

impl FakeMemory {
    pub fn new() -> Self {
        Self {
            next: RefCell::new(vec![]),
        }
    }
    pub fn push_byte(&mut self, n: u8) {
        self.next.borrow_mut().push(FakeMemoryItem::Byte(n))
    }
    pub fn push_word(&mut self, n: u32) {
        self.next.borrow_mut().push(FakeMemoryItem::Word(n))
    }
    pub fn push_double(&mut self, n: u64) {
        self.next.borrow_mut().push(FakeMemoryItem::Double(n))
    }
    pub fn queue_size(&self) -> usize {
        self.next.borrow().len()
    }
}

impl Memory for FakeMemory {
    fn write_b(&mut self, _offset: u64, _value: u8) {}
    fn read_b(&self, _offset: u64) -> u8 {
        match self.next.borrow_mut().pop() {
            Some(FakeMemoryItem::Byte(n)) => n,
            n => panic!("Expected read fake byte, but was: {:?}", n),
        }
    }
    fn read_d(&self, _offset: u64) -> u64 {
        match self.next.borrow_mut().pop() {
            Some(FakeMemoryItem::Double(n)) => n,
            n => panic!("Expected read fake word, but was: {:?}", n),
        }
    }
    fn read_w(&self, _offset: u64) -> u32 {
        match self.next.borrow_mut().pop() {
            Some(FakeMemoryItem::Word(n)) => n,
            n => panic!("Expected read fake word, but was: {:?}", n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_env_logger;

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
