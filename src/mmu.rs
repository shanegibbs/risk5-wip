use crate::bitfield::{Mstatus, PageTableEntry, PhysicalAddress, VirtualAddress};
use crate::Memory;
use std::fmt;

const INSN_CACHE_SIZE: usize = 10_000;

pub(crate) struct Mmu<M> {
    mem: M,
    prv: u64,
    insn_prv: u64,
    sv39: bool,
    asid: u16,
    ppn: u64,
    cache: Vec<(u64, u64)>,
    insn_cache: [(u64, u32); INSN_CACHE_SIZE],
    hit: u64,
    miss: u64,
}

fn new_cache() -> Vec<(u64, u64)> {
    vec![(0, 0); 4000]
}

impl<M> Mmu<M> {
    pub fn new(m: M) -> Self {
        Self {
            mem: m,
            prv: 3,
            insn_prv: 3,
            sv39: false,
            asid: 0,
            ppn: 0,
            cache: new_cache(),
            insn_cache: [(0, 0); INSN_CACHE_SIZE],
            hit: 0,
            miss: 0,
        }
    }

    pub fn mem(&self) -> &M {
        &self.mem
    }

    pub fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }

    pub fn flush_cache(&mut self) {
        self.insn_cache = [(0, 0); INSN_CACHE_SIZE];
    }

    pub fn set_prv(&mut self, prv: u64, mstatus: &Mstatus) {
        self.flush_cache();
        self.insn_prv = prv;
        self.prv = if mstatus.memory_privilege() == 1 {
            mstatus.machine_previous_privilege()
        } else {
            prv
        };
        trace!("MMU prv set to {}/{}", self.prv, self.insn_prv);
    }

    pub fn bare(&self) -> &M {
        &self.mem
    }
    pub fn bare_mut(&mut self) -> &mut M {
        &mut self.mem
    }

    pub fn set_bare_mode(&mut self) {
        trace!("Setting bare mode");
        self.flush_cache();
        self.sv39 = false;
        self.asid = 0;
        self.ppn = 0;
    }

    pub fn set_page_mode(&mut self, asid: u16, ppn: u64) {
        trace!("Setting sv39 mode asid=0x{:x} ppn={:x}", asid, ppn);
        self.flush_cache();
        self.sv39 = true;
        self.asid = asid;
        self.ppn = ppn;
    }
}

macro_rules! mem {
    ($self:expr, $func:ident, $prv:expr, $addr:expr) => {{
        let addr = match $self.translate($addr, $prv) {
            Ok(a) => a,
            Err(_) => {
                debug!("Page-fault on load");
                return Err(());
            }
        };
        let val = $self.mem.$func(addr);
        // if addr >= 0x80009000 && addr < 0x80009016 {
        //     debug!(
        //         "Doing htif {} at 0x{:x}: 0x{:x}",
        //         stringify!($func),
        //         addr,
        //         val
        //     );
        // }
        Ok(val)
    }};
    ($self:expr, $func:ident, $prv:expr, $addr:expr, $val:expr) => {{
        let addr = match $self.translate($addr, $prv) {
            Ok(a) => a,
            Err(_) => {
                debug!("Page-fault on store");
                return Err(());
            }
        };
        // if addr >= 0x80009000 && addr < 0x80009016 {
        //     debug!(
        //         "Doing htif {} at 0x{:x}: 0x{:x}",
        //         stringify!($func),
        //         addr,
        //         $val
        //     );
        // }
        Ok($self.mem.$func(addr, $val))
    }};
}

impl<M: Memory> Mmu<M> {
    fn translate(&mut self, offset: u64, prv: u64) -> Result<u64, ()> {
        if !self.sv39 || prv == 3 {
            return Ok(offset);
        }

        let vpage = offset >> 12;
        let cache_idx = (vpage as usize) % self.cache.len();
        let (cache_vpage, cache_ppage) = unsafe { self.cache.get_unchecked_mut(cache_idx) };
        if *cache_vpage == vpage {
            // self.hit += 1;
            info!("Page hit");
            // return Ok(*cache_ppage + (offset & 0xfff));
        }
        info!("Page miss");

        // self.miss += 1;
        // error!(
        //     "{} {}, {}",
        //     self.hit,
        //     self.miss,
        //     self.hit as f32 / (self.hit as f32 + self.miss as f32)
        // );

        trace!(
            "Translating offset 0x{:x} with asid=0x{:x}, ppn=0x{:x} and prv=0x{:x}",
            offset,
            self.asid,
            self.ppn,
            prv,
        );

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
            trace!("Read PTE at level {}: 0x{:x}", i, pte_val);

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
            i -= 1;
        };

        let mut pa: PhysicalAddress = 0.into();

        if i > 0 {
            // superpage translation
            for n in (0..i).rev() {
                let ppn = va.virtual_page_number(n);
                pa.set_physical_page_number_arr(n, ppn);
                trace!("Superpage: Set PPN {} on PA from VPN PPN field {}", n, n);
            }
        }

        for n in (i..levels).rev() {
            let ppn = pte.physical_page_number_arr(n);
            pa.set_physical_page_number_arr(n, ppn);
            trace!("Page: Set PPN {} on PA from PTE PPN field {}", n, n);
        }

        pa.set_offset(va.offset());
        let pa = pa.into();
        trace!("Translated to PA 0x{:x}", pa);

        *cache_vpage = vpage;
        *cache_ppage = pa & !0xfff;

        Ok(pa)
    }

    #[inline(never)]
    pub fn read_insn(&mut self, pc: u64) -> Result<u32, ()> {
        let cache_idx = ((pc >> 2) as usize) % INSN_CACHE_SIZE;
        {
            let (cpc, cinsn) = unsafe { self.insn_cache.get_unchecked(cache_idx) };
            if *cpc == pc {
                info!("pc hit");
                return Ok(*cinsn);
            }
        }
        info!("pc miss");

        let addr = match self.translate(pc, self.insn_prv) {
            Ok(a) => a,
            Err(_) => {
                debug!("Page-fault on load");
                return Err(());
            }
        };
        let val = self.mem.read_w(addr);

        let (cpc, cinsn) = unsafe { self.insn_cache.get_unchecked_mut(cache_idx) };
        *cpc = pc;
        *cinsn = val;
        Ok(val)
    }

    #[inline(never)]
    pub fn read_b(&mut self, offset: u64) -> Result<u8, ()> {
        mem!(self, read_b, self.prv, offset)
    }

    #[inline(never)]
    pub fn read_h(&mut self, offset: u64) -> Result<u16, ()> {
        mem!(self, read_h, self.prv, offset)
    }

    #[inline(never)]
    pub fn read_w(&mut self, offset: u64) -> Result<u32, ()> {
        mem!(self, read_w, self.prv, offset)
    }

    #[inline(never)]
    pub fn read_d(&mut self, offset: u64) -> Result<u64, ()> {
        mem!(self, read_d, self.prv, offset)
    }

    #[inline(never)]
    pub fn write_b(&mut self, offset: u64, value: u8) -> Result<(), ()> {
        mem!(self, write_b, self.prv, offset, value)
    }

    #[inline(never)]
    pub fn write_h(&mut self, offset: u64, value: u16) -> Result<(), ()> {
        mem!(self, write_h, self.prv, offset, value)
    }

    #[inline(never)]
    pub fn write_w(&mut self, offset: u64, value: u32) -> Result<(), ()> {
        mem!(self, write_w, self.prv, offset, value)
    }

    #[inline(never)]
    pub fn write_d(&mut self, offset: u64, value: u64) -> Result<(), ()> {
        mem!(self, write_d, self.prv, offset, value)
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
        assert_eq!(
            mmu.translate(0xffffffe0000000c0, 0).expect("ok"),
            0x802000c0
        );

        let mut mmu = Mmu::new({
            let mut mem = FakeMemory::new();
            mem.push_read(FakeMemoryItem::Double(0x80687010, 0x201800cf));
            mem.push_read(FakeMemoryItem::Double(0x80707c00, 0x201a1c01));
            mem
        });
        mmu.set_page_mode(0, 0x80707);
        assert_eq!(
            mmu.translate(0xffffffe000464440, 0).expect("ok"),
            0x80664440
        );

        let mut mmu = Mmu::new({
            let mut mem = FakeMemory::new();
            mem.push_read(FakeMemoryItem::Double(0x80687000, 0x200800cf));
            mem.push_read(FakeMemoryItem::Double(0x80707c00, 0x201a1c01));
            mem
        });
        mmu.set_page_mode(0, 0x80707);

        let expected = 0x80202df8;
        let actual = mmu.translate(0xffffffe000002df8, 0).expect("ok");

        trace!("Actual   0x{:16x}", actual);
        trace!("Expected 0x{:16x}", expected);
        trace!("Actual   {:64b}", actual);
        trace!("Expected {:64b}", expected);

        assert_eq!(actual, expected);
    }
}

use crate::bitfield::Satp;
use crate::logrunner::RestorableState;

impl<'a, M> Into<Mmu<M>> for RestorableState<'a, M> {
    fn into(self) -> Mmu<M> {
        let satp: Satp = self.state.satp.into();
        let mstatus = self.state.mstatus.into();
        let mut mmu = Mmu {
            mem: self.memory,
            prv: 0,
            insn_prv: 0,
            sv39: satp.mode() == 8,
            asid: satp.asid() as u16,
            ppn: satp.ppn() as u64,
            cache: vec![(0, 0)],
            insn_cache: [(0, 0); INSN_CACHE_SIZE],
            hit: 0,
            miss: 0,
        };
        mmu.set_prv(self.state.prv, &mstatus);
        mmu
    }
}
