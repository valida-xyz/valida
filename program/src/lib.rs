#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::iter;
use valida_machine::{Chip, Machine, PrimeField, ProgramROM};

use p3_matrix::dense::RowMajorMatrix;

pub mod columns;
pub mod stark;

#[derive(Default)]
pub struct ProgramChip<F> {
    program_rom: ProgramROM<F>,
    pub counts: Vec<u32>,
}

impl<F> ProgramChip<F>
where
    F: PrimeField,
{
    pub fn from_program_rom(program_rom: ProgramROM<F>) -> Self {
        let counts = vec![0; program_rom.0.len()];
        Self {
            program_rom,
            counts,
        }
    }
}

impl<F, M> Chip<M> for ProgramChip<F>
where
    F: PrimeField,
    M: Machine<F = F>,
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
}

pub trait MachineWithProgramChip: Machine {
    fn program(&self) -> &ProgramChip<Self::F>;

    fn program_mut(&mut self) -> &mut ProgramChip<Self::F>;

    /// Read a word from the program code, and update the associated counter.
    fn read_word(&mut self, index: usize) {
        assert!(index < self.program().program_rom.0.len());
        self.program_mut().counts[index] += 1;
    }
}
