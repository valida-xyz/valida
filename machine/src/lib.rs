#![feature(generic_const_exprs)]

extern crate alloc;

extern crate self as valida_machine;

use core::ops::{Index, IndexMut};
pub use p3_field::field::{Field, Field32};
use p3_mersenne_31::Mersenne31 as Fp;

pub mod __internal;
pub mod chip;
pub mod config;
pub mod lookup;
pub mod proof;

pub use chip::Chip;

pub const OPERAND_ELEMENTS: usize = 5;
pub const INSTRUCTION_ELEMENTS: usize = OPERAND_ELEMENTS + 1;
pub const CPU_MEMORY_CHANNELS: usize = 3;

pub const MEMORY_CELL_BYTES: usize = 4;

#[derive(Copy, Clone, Default)]
pub struct Word<F>(pub [F; MEMORY_CELL_BYTES]);

pub trait Addressable<F: Copy>: Copy + From<u32> + From<Word<F>> {}

pub struct InstructionWord<F> {
    pub opcode: u32,
    pub operands: Operands<F>,
}

pub struct ProgramState<F> {
    pub pc: F,
    pub fp: F,
}

pub struct ProgramROM<F>(Vec<InstructionWord<F>>);

impl<F: Field32> ProgramROM<F> {
    pub fn get_instruction(&self, pc: F) -> &InstructionWord<F> {
        &self.0[pc.as_canonical_u32() as usize]
    }
}

#[derive(Copy, Clone, Default)]
pub struct Operands<F>([F; 5]);

impl<F: Copy> Operands<F> {
    pub fn a(&self) -> F {
        self.0[0]
    }
    pub fn b(&self) -> F {
        self.0[1]
    }
    pub fn c(&self) -> F {
        self.0[2]
    }
    pub fn d(&self) -> F {
        self.0[3]
    }
    pub fn e(&self) -> F {
        self.0[4]
    }
    pub fn is_imm(&self) -> F {
        self.0[4]
    }
}

impl<F> From<[F; MEMORY_CELL_BYTES]> for Word<F> {
    fn from(bytes: [F; MEMORY_CELL_BYTES]) -> Self {
        Self(bytes)
    }
}

impl From<Word<Fp>> for Fp {
    fn from(word: Word<Fp>) -> Self {
        todo!()
    }
}

impl<T> Index<usize> for Word<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for Word<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<F: Field> From<F> for Word<F> {
    fn from(bytes: F) -> Self {
        Self([F::ZERO, F::ZERO, F::ZERO, bytes])
    }
}

impl<F> IntoIterator for Word<F> {
    type Item = F;
    type IntoIter = core::array::IntoIter<F, MEMORY_CELL_BYTES>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<F> PartialEq for Word<F>
where
    F: Field,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
    }
}

impl<F> Eq for Word<F> where F: Field {}

impl<F> Into<u32> for Word<F> {
    fn into(self) -> u32 {
        todo!()
    }
}

impl<F> Into<[F; MEMORY_CELL_BYTES]> for Word<F> {
    fn into(self) -> [F; MEMORY_CELL_BYTES] {
        self.0
    }
}

pub trait Machine {
    type F: Field;
    fn run(&mut self, initial_state: ProgramState<Self::F>, program: ProgramROM<Self::F>);
    fn prove(&self);
    fn verify();
}
