// TODO: Convert memory from big endian to little endian

extern crate alloc;
extern crate self as valida_machine;

pub use p3_field::{AbstractField, Field, PrimeField, PrimeField32, PrimeField64};

pub mod __internal;
pub mod chip;
pub mod config;
pub mod core;
pub mod lookup;
pub mod proof;

pub use crate::core::Word;
pub use chip::{Chip, Instruction, Interaction};

pub const OPERAND_ELEMENTS: usize = 5;
pub const INSTRUCTION_ELEMENTS: usize = OPERAND_ELEMENTS + 1;
pub const CPU_MEMORY_CHANNELS: usize = 3;
pub const MEMORY_CELL_BYTES: usize = 4;
pub const LOOKUP_DEGREE_BOUND: usize = 3;

pub struct InstructionWord<F> {
    pub opcode: u32,
    pub operands: Operands<F>,
}

pub struct ProgramROM<F>(Vec<InstructionWord<F>>);

impl<F> ProgramROM<F> {
    pub fn new(instructions: Vec<InstructionWord<F>>) -> Self {
        Self(instructions)
    }

    pub fn get_instruction(&self, pc: u32) -> &InstructionWord<F> {
        &self.0[pc as usize]
    }
}

#[derive(Copy, Clone, Default)]
pub struct Operands<F>(pub [F; 5]);

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

impl<F: PrimeField> Operands<F> {
    pub fn from_i32_slice(slice: &[i32]) -> Self {
        let mut operands = [F::ZERO; 5];
        for (i, &operand) in slice.iter().enumerate() {
            let abs = F::from_canonical_u32(operand.abs() as u32);
            operands[i] = if operand < 0 { -abs } else { abs };
        }
        Self(operands)
    }
}

pub trait Machine {
    type F: PrimeField64;
    fn run(&mut self, program: ProgramROM<i32>);
    fn prove(&self);
    fn verify();
}
