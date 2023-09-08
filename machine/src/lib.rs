#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate self as valida_machine;

use alloc::vec::Vec;

pub use crate::core::Word;
pub use chip::{BusArgument, Chip, Interaction, InteractionType, ValidaAirBuilder};

use crate::config::StarkConfig;
use crate::proof::MachineProof;
pub use p3_field::{
    AbstractExtensionField, AbstractField, ExtensionField, Field, PrimeField, PrimeField32,
    PrimeField64,
};

pub mod __internal;
pub mod chip;
pub mod config;
pub mod core;
pub mod proof;

pub const OPERAND_ELEMENTS: usize = 5;
pub const INSTRUCTION_ELEMENTS: usize = OPERAND_ELEMENTS + 1;
pub const CPU_MEMORY_CHANNELS: usize = 3;
pub const MEMORY_CELL_BYTES: usize = 4;
pub const LOOKUP_DEGREE_BOUND: usize = 3;

pub trait Instruction<M: Machine> {
    const OPCODE: u32;

    fn execute(state: &mut M, ops: Operands<i32>);
}

#[derive(Copy, Clone, Default)]
pub struct InstructionWord<F> {
    pub opcode: u32,
    pub operands: Operands<F>,
}

impl InstructionWord<i32> {
    pub fn flatten<F: PrimeField32>(&self) -> [F; INSTRUCTION_ELEMENTS] {
        let mut result = [F::default(); INSTRUCTION_ELEMENTS];
        result[0] = F::from_canonical_u32(self.opcode);
        self.operands.0.into_iter().enumerate().for_each(|(i, x)| {
            result[i] = if x >= 0 {
                F::from_canonical_u32(x as u32)
            } else {
                F::from_wrapped_u32((x as i64 + F::ORDER_U32 as i64) as u32)
            };
        });
        result
    }
}

#[derive(Copy, Clone, Default)]
pub struct Operands<F>(pub [F; OPERAND_ELEMENTS]);

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
    pub fn imm32(&self) -> Word<F> {
        Word([self.0[1], self.0[2], self.0[3], self.0[4]])
    }
}

impl<F: PrimeField> Operands<F> {
    pub fn from_i32_slice(slice: &[i32]) -> Self {
        let mut operands = [F::ZERO; OPERAND_ELEMENTS];
        for (i, &operand) in slice.iter().enumerate() {
            let abs = F::from_canonical_u32(operand.abs() as u32);
            operands[i] = if operand < 0 { -abs } else { abs };
        }
        Self(operands)
    }
}

#[derive(Default, Clone)]
pub struct ProgramROM<F>(pub Vec<InstructionWord<F>>);

impl<F> ProgramROM<F> {
    pub fn new(instructions: Vec<InstructionWord<F>>) -> Self {
        Self(instructions)
    }

    pub fn get_instruction(&self, pc: u32) -> &InstructionWord<F> {
        &self.0[pc as usize]
    }
}

pub trait Machine {
    type F: PrimeField64;
    type EF: ExtensionField<Self::F>;

    fn run(&mut self, program: &ProgramROM<i32>);

    fn prove<SC>(&self, config: &SC) -> MachineProof<SC>
    where
        SC: StarkConfig<Val = Self::F, Challenge = Self::EF>;

    fn verify<SC>(proof: &MachineProof<SC>) -> Result<(), ()>
    where
        SC: StarkConfig<Val = Self::F, Challenge = Self::EF>;
}
