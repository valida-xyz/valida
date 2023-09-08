#![no_std]

extern crate alloc;

use crate::columns::{COL_MAP, PREPROCESSED_COL_MAP};
use alloc::vec;
use alloc::vec::Vec;
use core::iter;
use valida_bus::MachineWithProgramBus;
use valida_machine::{Chip, Interaction, Machine, PrimeField, ProgramROM};

use p3_air::VirtualPairCol;
use p3_matrix::dense::RowMajorMatrix;

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

impl<F, M> Chip<M> for ProgramChip
where
    F: PrimeField,
    M: MachineWithProgramBus<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let n = self.program_rom.0.len();
        let col = self
            .counts
            .iter()
            .map(|c| F::from_canonical_u32(*c))
            .chain(iter::repeat(F::ZERO))
            .take(n.next_power_of_two())
            .collect();
        RowMajorMatrix::new(col, 1)
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<F>> {
        let pc = VirtualPairCol::single_main(COL_MAP.counter);
        let opcode = VirtualPairCol::single_preprocessed(PREPROCESSED_COL_MAP.opcode);
        let mut fields = vec![pc, opcode];
        fields.extend(
            PREPROCESSED_COL_MAP
                .operands
                .0
                .iter()
                .map(|op| VirtualPairCol::single_preprocessed(*op)),
        );
        let receives = Interaction {
            fields,
            count: VirtualPairCol::single_main(COL_MAP.multiplicity),
            argument_index: machine.program_bus(),
        };
        vec![receives]
    }
}

pub trait MachineWithProgramChip: Machine {
    fn program(&self) -> &ProgramChip;

    fn program_mut(&mut self) -> &mut ProgramChip;

    /// Read a word from the program code, and update the associated counter.
    fn read_word(&mut self, index: usize) {
        assert!(index < self.program().program_rom.0.len());
        self.program_mut().counts[index] += 1;
    }
}
