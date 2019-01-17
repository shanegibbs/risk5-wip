use crate::Memory;
use std::collections::HashMap;

pub(crate) struct ByteMap {
    pub data: HashMap<u64, u8>,
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
        let value = *self.data.get(&offset).expect("No memory here");
        trace!("Loaded 0x{:x} from 0x{:x}", value, offset);
        value
    }

    fn write_b(&mut self, offset: u64, value: u8) {
        // trace!("Saving 0x{:x} to 0x{:x}", value, offset);
        self.data.insert(offset, value);
    }
}
