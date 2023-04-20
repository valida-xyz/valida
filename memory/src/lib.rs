#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use p3_field::field::Field;

pub mod columns;

pub const MEMORY_CELL_BYTES: usize = 4;

#[derive(Copy, Clone)]
pub struct Word<F: Copy>([F; MEMORY_CELL_BYTES]);

impl<F: Copy> Into<u32> for Word<F> {
    fn into(self) -> u32 {
        todo!()
    }
}

pub struct Memory<F: Copy> {
    cells: BTreeMap<u32, Word<F>>,
}

pub struct MemoryLog<F> {
    pub address: F,
    pub value: F,
}

impl<F: Copy> Memory<F> {
    pub fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
        }
    }

    pub fn read(&self, address: u32) -> Word<F> {
        self.cells.get(&address).copied().unwrap()
    }

    pub fn write(&mut self, address: u32, value: Word<F>) {
        self.cells.insert(address, value);
    }
}
