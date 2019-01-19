use crate::bitfield::{PageTableEntry, PhysicalAddress, VirtualAddress};
use crate::Memory;
use std::fmt;

pub struct Mmu<M> {
    mem: M,
    sv39: bool,
    asid: u16,
    ppn: u64,
}

impl<M> Mmu<M> {
    pub fn new(m: M) -> Self {
        Self {
            mem: m,
            sv39: false,
            asid: 0,
            ppn: 0,
        }
    }

    pub fn mem(&self) -> &M {
        &self.mem
    }

    pub fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }

    pub fn bare(&self) -> &M {
        &self.mem
    }
    pub fn bare_mut(&mut self) -> &mut M {
        &mut self.mem
    }

    pub fn set_bare_mode(&mut self) {
        trace!("Setting bare mode");
        self.sv39 = false;
        self.asid = 0;
        self.ppn = 0;
    }

    pub fn set_page_mode(&mut self, asid: u16, ppn: u64) {
        trace!("Setting sv39 mode asid=0x{:x} ppn={:x}", asid, ppn);
        self.sv39 = true;
        self.asid = asid;
        self.ppn = ppn;
    }
}

macro_rules! mem {
    ($self:expr, $func:ident, $addr:expr) => {{
        let addr = match $self.translate($addr) {
            Ok(a) => a,
            Err(_) => {
                debug!("Page-fault on load");
                return Err(());
            }
        };
        let val = $self.mem.$func(addr);
        if addr >= 0x80009000 && addr < 0x80009016 {
            warn!(
                "Doing htif {} at 0x{:x}: 0x{:x}",
                stringify!($func),
                addr,
                val
            );
        }
        Ok(val)
    }};
    ($self:expr, $func:ident, $addr:expr, $val:expr) => {{
        let addr = match $self.translate($addr) {
            Ok(a) => a,
            Err(_) => {
                debug!("Page-fault on store");
                return Err(());
            }
        };
        if addr >= 0x80009000 && addr < 0x80009016 {
            warn!(
                "Doing htif {} at 0x{:x}: 0x{:x}",
                stringify!($func),
                addr,
                $val
            );
        }
        Ok($self.mem.$func(addr, $val))
    }};
}

impl<M: Memory> Mmu<M> {
    fn translate(&self, offset: u64) -> Result<u64, ()> {
        if !self.sv39 {
            return Ok(offset);
        }

        let pagesize = 4096;
        let levels = 3;
        let ptesize = 8;

        let va: VirtualAddress = offset.into();

        // step 1
        let mut a = self.ppn * pagesize;
        let mut i = levels - 1;

        let pte = loop {
            // step 2
            let pte_offset = a + (va.virtual_page_number(i) * ptesize);

            let pte_val = self.mem.read_d(pte_offset);

            let pte: PageTableEntry = pte_val.into();

            if !pte.v() || (!pte.r() && pte.w()) {
                // step 3. page-fault exception
                return Err(());
            }

            if pte.r() || pte.x() {
                // step 5
                break pte;
            }

            // pte is a pointer to the next page
            if i == 0 {
                panic!("sv39 step 4 page-fault")
            }

            // step down a level
            a = pte.physical_page_number() * pagesize;
            i = i - 1;
        };

        let mut pa: PhysicalAddress = 0.into();

        if i > 0 {
            // superpage translation
            for n in (0..i).rev() {
                let ppn = va.virtual_page_number(i);
                pa.set_physical_page_number_arr(n, ppn);
            }
        }

        for n in (i..levels).rev() {
            let ppn = pte.physical_page_number_arr(n);
            pa.set_physical_page_number_arr(n, ppn);
        }

        pa.set_offset(va.offset());

        Ok(pa.into())
    }

    pub fn read_b(&self, offset: u64) -> Result<u8, ()> {
        mem!(self, read_b, offset)
    }

    pub fn read_h(&self, offset: u64) -> Result<u16, ()> {
        mem!(self, read_h, offset)
    }

    pub fn read_w(&self, offset: u64) -> Result<u32, ()> {
        mem!(self, read_w, offset)
    }

    pub fn read_d(&self, offset: u64) -> Result<u64, ()> {
        mem!(self, read_d, offset)
    }

    pub fn write_b(&mut self, offset: u64, value: u8) -> Result<(), ()> {
        mem!(self, write_b, offset, value)
    }

    pub fn write_h(&mut self, offset: u64, value: u16) -> Result<(), ()> {
        mem!(self, write_h, offset, value)
    }

    pub fn write_w(&mut self, offset: u64, value: u32) -> Result<(), ()> {
        mem!(self, write_w, offset, value)
    }

    pub fn write_d(&mut self, offset: u64, value: u64) -> Result<(), ()> {
        mem!(self, write_d, offset, value)
    }
}

impl<M> fmt::Debug for Mmu<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mmu")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::{FakeMemory, FakeMemoryItem};

    #[test]
    fn first_linux_page_translation() {
        let mut mmu = Mmu::new({
            let mut mem = FakeMemory::new();
            mem.push_read(FakeMemoryItem::Double(0x8021c000, 0x200800cf));
            mem.push_read(FakeMemoryItem::Double(0x8021dc00, 0x20087001));
            mem
        });
        mmu.set_page_mode(0, 0x8021d);
        assert_eq!(mmu.translate(0xffffffe0000000c0).expect("ok"), 0x802000c0);

        let mut mmu = Mmu::new({
            let mut mem = FakeMemory::new();
            mem.push_read(FakeMemoryItem::Double(0x80687010, 0x201800cf));
            mem.push_read(FakeMemoryItem::Double(0x80707c00, 0x201a1c01));
            mem
        });
        mmu.set_page_mode(0, 0x80707);
        assert_eq!(mmu.translate(0xffffffe000464440).expect("ok"), 0x80602440);
    }
}
