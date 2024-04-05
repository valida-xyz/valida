extern crate alloc;

pub mod columns;
pub mod stark;

use alloc::borrow::Borrow;
use alloc::vec::Vec;
use columns::{Mul64Cols, MUL64_COL_MAP, NUM_MUL64_COLS, Word64};
use valida_machine::{Chip, Interaction, Machine, Word, StarkConfig};

use p3_field::{AbstractField, Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;

pub struct Operation {
    pub inputs: (Word<u8>, Word<u8>),
    pub output: Word64<u8>,
}

pub struct Mul64Chip {
    pub operations: Vec<Operation>,
}

impl<M, SC> Chip<M, SC> for Mul64Chip
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        todo!()
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        todo!()
    }
}

impl Mul64Chip {
    fn op_to_row<F>(&self, op: &Operation, cosl: &mut Mul64Cols<F>)
    where
        F: PrimeField,
    {
        todo!()
    }

    fn set_cols<F>(&self, a: &Word<u8>, b: &Word<u8>, c: &Word64<u8>, cols: &mut Mul64Cols<F>)
    where
        F: PrimeField,
    {
        todo!()
    }
}

pub trait MachineWithMul64Chip<F: Field>: Machine<F> {
    fn mul_u64(&self) -> &Mul64Chip;
    fn mul_u64_mut(&mut self) -> &mut Mul64Chip;
}
