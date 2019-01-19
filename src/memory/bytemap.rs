use crate::Memory;

pub(crate) struct ByteMap {
    pub data: Vec<(u64, u8)>,
}

impl ByteMap {
    pub fn new() -> Self {
        ByteMap { data: vec![] }
    }

    pub fn clear(&mut self) {
        self.data.retain(|(addr, _)| *addr < 0x4096);
    }
}

impl Memory for ByteMap {
    fn read_b(&self, offset: u64) -> u8 {
        for (addr, value) in (&self.data).iter().rev() {
            if *addr == offset {
                trace!("Loaded 0x{:x}: 0x{:x}", offset, value);
                return *value;
            }
        }
        error!("No memory here at 0x{:x}", offset);
        0
    }

    fn write_b(&mut self, offset: u64, value: u8) {
        // trace!("Saving 0x{:x} to 0x{:x}", value, offset);
        self.data.push((offset, value));
    }
}
