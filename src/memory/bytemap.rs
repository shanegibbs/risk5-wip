use crate::Memory;

#[derive(Clone)]
pub(crate) struct ByteMap {
    pub persistent: Vec<(u64, u8)>,
    pub data: Vec<(u64, u8)>,
}

impl ByteMap {
    pub fn with_persistent(mut self, p: Vec<(u64, u8)>) -> Self {
        self.persistent = p;
        self
    }

    pub fn into_data(self) -> Vec<(u64, u8)> {
        self.data
    }

    pub fn _set_data(&mut self, data: Vec<(u64, u8)>) {
        self.data = data;
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for ByteMap {
    fn default() -> Self {
        ByteMap {
            persistent: vec![],
            data: vec![],
        }
    }
}

impl Memory for ByteMap {
    fn read_b(&mut self, offset: u64) -> u8 {
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
