extern crate alloc;

extern crate self as valida_machine;

use p3_field::field::Field;
use p3_mersenne_31::Mersenne31 as Fp;

pub mod __internal;
pub mod bus;
pub mod chip;
pub mod config;
pub mod constraint_consumer;
pub mod instruction;
pub mod proof;

pub use instruction::Instruction;

pub const OPERAND_ELEMENTS: usize = 5;
pub const INSTRUCTION_ELEMENTS: usize = OPERAND_ELEMENTS + 1;
pub const CPU_MEMORY_CHANNELS: usize = 3;

pub const MEMORY_CELL_BYTES: usize = 4;

#[derive(Copy, Clone, Default)]
pub struct Word<F>(pub [F; MEMORY_CELL_BYTES]);

#[derive(Copy, Clone, Default)]
pub struct InstructionWord<F>([F; INSTRUCTION_ELEMENTS]);

pub trait Addressable<F: Copy>: Copy + From<u32> + From<Word<F>> {}

#[derive(Default)]
pub struct Operands([Fp; 5]);

impl Operands {
    pub fn a(&self) -> Fp {
        self.0[0]
    }
    pub fn b(&self) -> Fp {
        self.0[1]
    }
    pub fn c(&self) -> Fp {
        self.0[2]
    }
    pub fn d(&self) -> Fp {
        self.0[3]
    }
    pub fn e(&self) -> Fp {
        self.0[4]
    }
    pub fn is_imm(&self) -> Fp {
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

impl<F: Field> From<F> for Word<F> {
    fn from(bytes: F) -> Self {
        Self([F::ZERO, F::ZERO, F::ZERO, bytes])
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
    fn run(&mut self);
    fn prove(&self);
    fn verify();
}
