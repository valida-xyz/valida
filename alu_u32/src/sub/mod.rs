extern crate alloc;

use alloc::vec::Vec;
use columns::{Sub32Cols, NUM_SUB_COLS};
use core::mem::transmute;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Instruction, Operands, Word, MEMORY_CELL_BYTES};

use p3_field::{Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use valida_machine::Chip;

pub mod columns;
mod stark;

#[derive(Clone)]
pub enum Operation<F> {
    Sub32(Word<F>, Word<F>, Word<F>),
}

#[derive(Default)]
pub struct Sub32Chip<F> {
    pub clock: F,
    pub operations: Vec<Operation<F>>,
}

impl<M> Chip<M> for Sub32Chip<M::F>
where
    M: MachineWithSub32Chip,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .iter()
            .cloned()
            .map(|op| self.op_to_row(op, machine))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_SUB_COLS)
    }
}

impl<F: Field> Sub32Chip<F> {
    fn op_to_row<M>(&self, op: Operation<F>, _machine: &M) -> [F; NUM_SUB_COLS]
    where
        M: MachineWithSub32Chip,
    {
        let mut row = [F::ZERO; NUM_SUB_COLS];
        let mut cols: &mut Sub32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Sub32(a, b, c) => {
                cols.input_1 = b;
                cols.input_2 = c;
                cols.output = a;
            }
        }
        row
    }
}

pub trait MachineWithSub32Chip: MachineWithCpuChip {
    fn sub_u32(&self) -> &Sub32Chip<Self::F>;
    fn sub_u32_mut(&mut self) -> &mut Sub32Chip<Self::F>;
}

instructions!(Sub32Instruction);

impl<F, M> Instruction<M> for Sub32Instruction
where
    F: PrimeField,
    M: MachineWithSub32Chip<F = F>,
{
    const OPCODE: u32 = 8;

    fn execute(state: &mut M, ops: Operands<F>) {
        todo!()
    }
}
