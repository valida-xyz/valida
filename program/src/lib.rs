#![no_std]

extern crate alloc;

use crate::columns::NUM_PROGRAM_COLS;
use alloc::vec;
use alloc::vec::Vec;
use valida_bus::MachineWithProgramBus;
use valida_machine::{Chip, Interaction, Machine, ProgramROM};
use valida_util::pad_to_power_of_two;

use p3_field::{AbstractField, Field};
use p3_matrix::dense::RowMajorMatrix;
use valida_machine::StarkConfig;

pub mod columns;
pub mod stark;

#[derive(Default)]
pub struct ProgramChip {
    program_rom: ProgramROM<i32>,
    pub counts: Vec<u32>,
}

impl ProgramChip {
    pub fn set_program_rom(&mut self, rom: &ProgramROM<i32>) {
        let counts = vec![0; rom.0.len()];
        self.program_rom = rom.clone();
        self.counts = counts;
    }
}

impl<M, SC> Chip<M, SC> for ProgramChip
where
    M: MachineWithProgramBus<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        let mut values = self
            .counts
            .iter()
            .map(|c| SC::Val::from_canonical_u32(*c))
            .collect();

        pad_to_power_of_two::<NUM_PROGRAM_COLS, SC::Val>(&mut values);

        RowMajorMatrix::new(values, NUM_PROGRAM_COLS)
    }

    fn global_receives(&self, _machine: &M) -> Vec<Interaction<SC::Val>> {
        // let pc = VirtualPairCol::single_preprocessed(PREPROCESSED_COL_MAP.pc);
        // let opcode = VirtualPairCol::single_preprocessed(PREPROCESSED_COL_MAP.opcode);
        // let mut fields = vec![pc, opcode];
        // fields.extend(
        //     PREPROCESSED_COL_MAP
        //         .operands
        //         .0
        //         .iter()
        //         .map(|op| VirtualPairCol::single_preprocessed(*op)),
        // );
        // let receives = Interaction {
        //     fields,
        //     count: VirtualPairCol::single_main(COL_MAP.multiplicity),
        //     argument_index: machine.program_bus(),
        // };
        // vec![receives]
        vec![]
    }
}

pub trait MachineWithProgramChip<F: Field>: Machine<F> {
    fn program(&self) -> &ProgramChip;

    fn program_mut(&mut self) -> &mut ProgramChip;

    /// Read a word from the program code, and update the associated counter.
    fn read_word(&mut self, index: usize) {
        assert!(index < self.program().program_rom.0.len());
        self.program_mut().counts[index] += 1;
    }
}
