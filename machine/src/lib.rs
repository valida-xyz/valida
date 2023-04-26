extern crate alloc;

extern crate self as valida_machine;

use p3_field::field::Field;

pub mod __internal;
pub mod bus;
pub mod chip;
pub mod config;
pub mod constraint_consumer;
pub mod instruction;
pub mod proof;

pub const MEMORY_CELL_BYTES: usize = 4;

#[derive(Copy, Clone, Default)]
pub struct Word<F>([F; MEMORY_CELL_BYTES]);

pub trait Addressable<F: Copy>: Copy + From<u32> + From<Word<F>> {}

impl<F: Copy> Into<u32> for Word<F> {
    fn into(self) -> u32 {
        todo!()
    }
}

impl<F: Copy> From<u32> for Word<F> {
    fn from(value: u32) -> Self {
        todo!()
    }
}

impl<F: Copy> Into<[F; MEMORY_CELL_BYTES]> for Word<F> {
    fn into(self) -> [F; MEMORY_CELL_BYTES] {
        self.0
    }
}

pub trait Machine {
    type F: Field;
    fn run(&mut self);
    fn prove(&self);
    fn verify();
}
