#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use p3_field::field::Field;

pub mod columns;

pub const MEMORY_CELL_BYTES: usize = 4;

pub struct Memory<F: Field> {
    cells: BTreeMap<F, [F; MEMORY_CELL_BYTES]>,
}

impl<F: Field> Memory<F> {
    pub fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
        }
    }
}
