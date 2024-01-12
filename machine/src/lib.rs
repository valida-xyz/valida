#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;
extern crate self as valida_machine;

use alloc::vec::Vec;

use byteorder::{ByteOrder, LittleEndian};

pub use crate::core::Word;
pub use chip::{BusArgument, Chip, Interaction, InteractionType, ValidaAirBuilder};
use p3_matrix::dense::{RowMajorMatrix};

use p3_air::Air;
use crate::proof::MachineProof;
pub use p3_field::{
    AbstractExtensionField, AbstractField, ExtensionField, Field, PrimeField, PrimeField64,
};
use p3_uni_stark::{
   ProverConstraintFolder,
    StarkConfig, SymbolicAirBuilder, Commitments, Proof,  ProverData
};
// TODO: some are also re-exported, so they shouldn't be pub?
pub mod __internal;
mod advice;
pub mod chip;
pub mod config;
pub mod core;
pub mod proof;

pub use advice::*;
pub use chip::*;
pub use core::*;

pub const OPERAND_ELEMENTS: usize = 5;
pub const INSTRUCTION_ELEMENTS: usize = OPERAND_ELEMENTS + 1;
pub const CPU_MEMORY_CHANNELS: usize = 3;
pub const MEMORY_CELL_BYTES: usize = 4;
pub const LOOKUP_DEGREE_BOUND: usize = 3;

pub trait Instruction<M: Machine> {
    const OPCODE: u32;

    fn execute(state: &mut M, ops: Operands<i32>);

    fn execute_with_advice<Adv: AdviceProvider>(
        state: &mut M,
        ops: Operands<i32>,
        _advice: &mut Adv,
    ) {
        Self::execute(state, ops)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct InstructionWord<F> {
    pub opcode: u32,
    pub operands: Operands<F>,
}

impl InstructionWord<i32> {
    pub fn flatten<F: PrimeField64>(&self) -> [F; INSTRUCTION_ELEMENTS] {
        let mut result = [F::default(); INSTRUCTION_ELEMENTS];
        result[0] = F::from_canonical_u32(self.opcode);
        result[1..].copy_from_slice(&Operands::<F>::from_i32_slice(&self.operands.0).0);
        result
    }
}

#[derive(Copy, Clone, Default, Debug)]
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
        let mut operands = [F::zero(); OPERAND_ELEMENTS];
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

impl ProgramROM<i32> {
    pub fn from_machine_code(mc: &[u8]) -> Self {
        let mut instructions = Vec::new();
        for chunk in mc.chunks_exact(INSTRUCTION_ELEMENTS * 4) {
            instructions.push(InstructionWord {
                opcode: LittleEndian::read_u32(&chunk[0..4]),
                operands: Operands([
                    LittleEndian::read_i32(&chunk[4..8]),
                    LittleEndian::read_i32(&chunk[8..12]),
                    LittleEndian::read_i32(&chunk[12..16]),
                    LittleEndian::read_i32(&chunk[16..20]),
                    LittleEndian::read_i32(&chunk[20..24]),
                ]),
            });
        }
        Self(instructions)
    }

    #[cfg(feature = "std")]
    pub fn from_file(filename: &str) -> std::io::Result<Self> {
        use byteorder::ReadBytesExt;
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        let mut instructions = Vec::new();

        while let Ok(opcode) = reader.read_u32::<LittleEndian>() {
            let mut operands_arr = [0i32; OPERAND_ELEMENTS];
            for i in 0..OPERAND_ELEMENTS {
                operands_arr[i] = reader.read_i32::<LittleEndian>()?;
            }
            let operands = Operands(operands_arr);
            instructions.push(InstructionWord { opcode, operands });
        }

        Ok(ProgramROM::new(instructions))
    }
}

pub trait Machine {
    type F: PrimeField64;
    type EF: ExtensionField<Self::F>;

    fn run<Adv: AdviceProvider>(&mut self, program: &ProgramROM<i32>, advice: &mut Adv);

    fn add_chip_trace<SC, A>(
        &self,
        config: &SC,
        challenger: &mut SC::Challenger,
        trace_commitments: &mut Vec<ProverData<SC>>,
        quotient_commitments: &mut Vec<ProverData<SC>>,
        log_degree: &mut Vec<usize>,
        log_quotient_degrees: &mut Vec<usize>,
	chip:&A,
	trace:RowMajorMatrix<<SC as StarkConfig>::Val>
    ) where
        SC: StarkConfig,
        A: Air<SymbolicAirBuilder<SC::Val>> + for<'a> Air<ProverConstraintFolder<'a, SC>>;

    fn prove<SC>(&self, config: &SC, challenger: &mut SC::Challenger) -> MachineProof<SC>
    where
        SC: StarkConfig<Val = Self::F, Challenge = Self::EF>;

    fn verify<SC>(proof: &MachineProof<SC>) -> Result<(), ()>
    where
        SC: StarkConfig<Val = Self::F, Challenge = Self::EF>;
}
