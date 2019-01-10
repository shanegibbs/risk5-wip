use crate::memory::Memory;
use std::fmt;

#[derive(Debug)]
pub enum FakeMemoryItem {
    Byte(u64, u8),
    Word(u64, u32),
    Double(u64, u64),
}

impl FakeMemoryItem {
    fn addr(&self) -> u64 {
        use self::FakeMemoryItem::*;
        match self {
            Byte(o, _) => *o,
            Word(o, _) => *o,
            Double(o, _) => *o,
        }
    }
}

use std::cell::RefCell;
pub struct FakeMemory {
    next_read: RefCell<Vec<FakeMemoryItem>>,
    next_write: RefCell<Vec<FakeMemoryItem>>,
}

impl fmt::Debug for FakeMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FakeMemory")
    }
}

impl FakeMemory {
    pub fn new() -> Self {
        Self {
            next_read: RefCell::new(vec![]),
            next_write: RefCell::new(vec![]),
        }
    }

    pub fn push_read<T>(&mut self, t: T)
    where
        T: Into<FakeMemoryItem>,
    {
        self.next_read.borrow_mut().push(t.into())
    }

    pub fn push_write<T>(&mut self, t: T)
    where
        T: Into<FakeMemoryItem>,
    {
        self.next_write.borrow_mut().push(t.into())
    }

    pub fn queue_size(&self) -> usize {
        self.next_read.borrow().len() + self.next_write.borrow().len()
    }

    pub fn trim(&mut self) {
        self.next_read
            .borrow_mut()
            .retain(|n| n.addr() != 0x80009000 && n.addr() != 0x80009008);
        self.next_write
            .borrow_mut()
            .retain(|n| n.addr() != 0x80009000 && n.addr() != 0x80009008);
    }
    pub fn reset(&mut self) {
        self.next_read = RefCell::new(vec![]);
        self.next_write = RefCell::new(vec![]);
    }
}

macro_rules! check {
    ($n:expr, $expected:expr, $actual:expr) => {{
        if $actual != $expected {
            error!(
                "invalid offset: 0x{:x} expecting: 0x{:x}",
                $actual, $expected
            );
            panic!("memory offset fail");
        }
        return $n;
    }};
}

macro_rules! check_write {
    ($offset:expr, $value:expr, $expected_offset:expr, $expected_value:expr) => {{
        let mut failed = false;

        if $offset != $expected_offset {
            error!(
                "invalid offset: 0x{:x} expecting: 0x{:x}",
                $offset, $expected_offset
            );
            failed = true;
        }

        if $value != $expected_value {
            error!(
                "invalid value: 0x{:x} expecting: 0x{:x}",
                $value, $expected_value
            );
            failed = true;
        }

        if failed {
            panic!("memory offset fail");
        }
    }};
}

impl Memory for FakeMemory {
    fn write_b(&mut self, offset: u64, value: u8) {
        match self.next_write.borrow_mut().pop() {
            Some(FakeMemoryItem::Byte(expected_offset, expected_value)) => {
                check_write!(offset, value, expected_offset, expected_value)
            }
            n => panic!("Expected write fake byte, but was: {:?}", n),
        }
    }

    fn write_w(&mut self, offset: u64, value: u32) {
        match self.next_write.borrow_mut().pop() {
            Some(FakeMemoryItem::Word(expected_offset, expected_value)) => {
                check_write!(offset, value, expected_offset, expected_value)
            }
            n => panic!("Expected write fake word, but was: {:?}", n),
        }
    }

    fn write_d(&mut self, offset: u64, value: u64) {
        match self.next_write.borrow_mut().pop() {
            Some(FakeMemoryItem::Double(expected_offset, expected_value)) => {
                check_write!(offset, value, expected_offset, expected_value)
            }
            n => panic!("Expected write fake double, but was: {:?}", n),
        }
    }

    fn read_b(&self, offset: u64) -> u8 {
        match self.next_read.borrow_mut().pop() {
            Some(FakeMemoryItem::Byte(addr, n)) => check!(n, addr, offset),
            n => panic!("Expected read fake byte, but was: {:?}", n),
        }
    }

    fn read_w(&self, offset: u64) -> u32 {
        match self.next_read.borrow_mut().pop() {
            Some(FakeMemoryItem::Word(addr, n)) => check!(n, addr, offset),
            n => panic!("Expected read fake word, but was: {:?}", n),
        }
    }

    fn read_d(&self, offset: u64) -> u64 {
        match self.next_read.borrow_mut().pop() {
            Some(FakeMemoryItem::Double(addr, n)) => check!(n, addr, offset),
            n => panic!(
                "Expected read fake double as 0x{:x}, but was: {:?}",
                offset, n
            ),
        }
    }
}
