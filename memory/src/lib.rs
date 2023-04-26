#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use valida_machine::Word;

pub mod columns;

pub struct Memory<F: Copy> {
    cells: BTreeMap<u32, Word<F>>,
}

pub struct MemoryLog<F: Copy> {
    pub address: Word<F>,
    pub value: Word<F>,
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
