extern crate alloc;

use alloc::vec::Vec;
use columns::NUM_SUB_COLS;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Instruction, Operands, Word, MEMORY_CELL_BYTES};

use p3_field::{AbstractField, PrimeField, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31 as Fp;
use valida_machine::Chip;

pub mod columns;
mod stark;

#[derive(Clone)]
pub enum Operation {
    Sub32,
}

#[derive(Default)]
pub struct Sub32Chip<F> {
    pub clock: F,
    pub operations: Vec<Operation>,
}

impl<M> Chip<M> for Sub32Chip<Fp>
where
    M: MachineWithSub32Chip<F = Fp>,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .iter()
            .cloned()
            .enumerate()
            .map(|(n, op)| self.op_to_row(op, machine))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_SUB_COLS)
    }
}

impl<F> Sub32Chip<F> {
    fn op_to_row<M>(&self, op: Operation, _machine: &M) -> [Fp; NUM_SUB_COLS]
    where
        M: MachineWithSub32Chip,
    {
        todo!()
    }
}

pub trait MachineWithSub32Chip: MachineWithCpuChip {
    fn add_u32(&self) -> &Sub32Chip<Self::F>;
    fn add_u32_mut(&mut self) -> &mut Sub32Chip<Self::F>;
}

instructions!(Sub32Instruction);

impl<M: MachineWithSub32Chip<F = Fp>> Instruction<M> for Sub32Instruction {
    const OPCODE: u32 = 8;

    fn execute(state: &mut M, ops: Operands<Fp>) {
        todo!()
    }
}
