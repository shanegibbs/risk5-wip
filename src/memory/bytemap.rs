use crate::Memory;

pub(crate) struct ByteMap {
    pub persistent: Vec<(u64, u8)>,
    pub data: Vec<(u64, u8)>,
}

impl ByteMap {
    pub fn new() -> Self {
        ByteMap {
            persistent: vec![],
            data: vec![],
        }
    }

    pub fn with_persistent(mut self, p: Vec<(u64, u8)>) -> Self {
        self.persistent = p;
        self
    }

    pub fn to_data(self) -> Vec<(u64, u8)> {
        self.data
    }

    pub fn clear(&mut self) {
        // self.data.retain(|(addr, _)| *addr < 0x4096);
        self.data.clear();
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
        for (addr, value) in (&self.persistent).iter().rev() {
            if *addr == offset {
                trace!("Loaded persistent 0x{:x}: 0x{:x}", offset, value);
                return *value;
            }
        }
        error!("No memory here at 0x{:x}", offset);
        0
    }

    fn write_b(&mut self, offset: u64, value: u8) {
        trace!("Storing 0x{:x}: 0x{:x}", offset, value);
        self.data.push((offset, value))
    }
}
