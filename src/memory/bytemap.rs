use std::collections::HashMap;
use crate::Memory;

pub(crate) struct ByteMap {
    data: HashMap<u64, u8>,
}

impl ByteMap {
    pub fn new() -> Self {
        ByteMap {
            data: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }
}

impl Memory for ByteMap {
    fn read_b(&self, offset: u64) -> u8 {
        *self.data.get(&offset).expect("No memory here")
    }

    fn write_b(&mut self, offset: u64, value: u8) {
        self.data.insert(offset, value);
    }
}
