use crate::Processor;
use std::fmt;

const CACHE_SIZE: usize = 100_000;

pub struct Matchers<M> {
    matchers: Vec<Matcher<M>>,
    hit: Vec<usize>,
    miss: Vec<usize>,
    matcher_cache: [(u32, usize); CACHE_SIZE],
}

/*
 * TODO
 *
 * - mmu/translate cache block and block offset of page
 * - marcher cache stats
 *
 */

impl<M> Matchers<M> {
    pub fn new(matchers: Vec<Matcher<M>>) -> Self {
        let len = matchers.len();
        Matchers {
            matchers,
            hit: vec![0; len],
            miss: vec![0; len],
            matcher_cache: [(0, 0); CACHE_SIZE],
        }
    }

    pub fn print(&self) {
        let mut results: Vec<_> = self.hit.iter().zip(&self.miss).enumerate().collect();
        results.sort_by(|a, b| {
            let a = (a.1).0 + (a.1).1;
            let b = (b.1).0 + (b.1).1;
            a.partial_cmp(&b).unwrap()
        });

        for (i, (hit, miss)) in results {
            println!("{:6} {:9} {:9} {:12}", i, hit, miss, hit + miss);
        }
    }

    #[inline(never)]
    pub fn find_for(&mut self, insn: u32) -> &Matcher<M> {
        let cache_idx = (insn as usize) % self.matcher_cache.len();
        let (cinsn, cmatcher) = unsafe { self.matcher_cache.get_unchecked_mut(cache_idx) };
        if *cinsn == insn {
            trace!("Insn hit");
            self.hit[*cmatcher] += 1;
            return unsafe { self.matchers.get_unchecked(*cmatcher) };
        }

        self.miss[*cmatcher] += 1;
        trace!("Insn miss");

        // for (i, matcher) in self.matchers.iter().enumerate() {
        //     if matcher.matches(insn) {
        //         cache.0 = insn;
        //         cache.1 = i;
        //         return matcher;
        //     }
        // }

        Self::find_for_slow(&self.matchers[..], insn, cinsn, cmatcher)
    }

    #[inline(never)]
    fn find_for_slow<'a, 'b: 'a>(
        matchers: &'b [Matcher<M>],
        insn: u32,
        cinsn: &mut u32,
        cmatcher: &mut usize,
    ) -> &'a Matcher<M> {
        *cinsn = insn;
        for (i, matcher) in matchers.iter().enumerate() {
            if matcher.matches(insn) {
                *cmatcher = i;
                return matcher;
            }
        }
        error!("no matched insn: 0x{:x}", insn);
        unreachable!("no matched insn");
    }
}

pub struct Matcher<M> {
    mask: u32,
    mtch: u32,
    exec: fn(&mut Processor<M>, u32),
}

impl<M> Matcher<M> {
    pub fn new(mask: u32, mtch: u32, exec: fn(&mut Processor<M>, u32)) -> Self {
        Self { mask, mtch, exec }
    }
    pub fn matches(&self, insn: u32) -> bool {
        insn & self.mask == self.mtch
    }
    pub fn exec(&self, p: &mut Processor<M>, insn: u32) {
        (self.exec)(p, insn)
    }
}

impl<M: fmt::Debug> fmt::Debug for Matcher<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Matcher")
    }
}
