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
        self.sv39 = false;
        self.asid = 0;
        self.ppn = 0;
    }

    pub fn set_page_mode(&mut self, asid: u16, ppn: u64) {
        self.sv39 = true;
        self.asid = asid;
        self.ppn = ppn;
    }
}

macro_rules! mem {
    ($self:expr, $func:ident, $addr:expr) => {{
        let addr = match $self.translate($addr) {
            Ok(a) => a,
            Err(_) => return Err(()),
        };
        Ok($self.mem.$func(addr))
    }};
    ($self:expr, $func:ident, $addr:expr, $val:expr) => {{
        let addr = match $self.translate($addr) {
            Ok(a) => a,
            Err(_) => return Err(()),
        };
        Ok($self.mem.$func(addr, $val))
    }};
}

impl<M: Memory> Mmu<M> {
    fn translate(&self, offset: u64) -> Result<u64, ()> {
        if !self.sv39 {
            return Ok(offset);
        }

        error!("Doing sv39");

        let pagesize = 4096;
        let levels = 3;
        let ptesize = 8;

        let va: VirtualAddress = offset.into();

        error!("offset=0x{:x}", va.offset());
        error!("vpn[0]=0x{:x}", va.virtual_page_number(0));
        error!("vpn[1]=0x{:x}", va.virtual_page_number(1));
        error!("vpn[2]=0x{:x}", va.virtual_page_number(2));

        // step 1
        let mut a = self.ppn * pagesize;
        let mut i = levels - 1;

        error!("i={},a=0x{:x}", i, a);

        let pte = loop {
            error!("-- Looking up level {}", i);

            // step 2
            let pte_offset = a + (va.virtual_page_number(i) * ptesize);
            error!("pte_offset=0x{:x}", pte_offset);

            let pte_val = self.mem.read_d(pte_offset);
            error!("pte_val=0x{:x}", pte_val);

            let pte: PageTableEntry = pte_val.into();
            error!("pte ppn=0x{:x}", pte.physical_page_number());

            if !pte.v() || (!pte.r() && pte.w()) {
                // step 3. page-fault exception
                error!("-- page-fault");
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

        error!("Using pte 0x{:x}", pte.val());

        let mut pa: PhysicalAddress = 0.into();
        pa.set_offset(pte.offset());
        error!("pa with offset 0x{:x}", pa.val());

        error!("i={}", i);
        for n in (i..levels - 1).rev() {
            let ppn = pte.physical_page_number_arr(n);
            error!("Using ppn 0x{:x}", ppn);
            pa.set_physical_page_number_arr(n, ppn);
        }

        error!("Using pa 0x{:x}", pa.val());

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
