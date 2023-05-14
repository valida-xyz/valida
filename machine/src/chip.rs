use crate::{Machine, Operands};

use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31 as Fp;

pub trait Chip<M> {
    type F;
    type FE;

    /// Generate the main trace for the chip given the provided machine.
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<Self::F>;

    /// Generate the permutation trace for the chip given the provided machine.
    fn generate_permutation_trace(
        &self,
        machine: &M,
        main_trace: RowMajorMatrix<Self::F>,
        random_elements: Vec<Self::FE>,
    ) -> RowMajorMatrix<Self::F>;
}

pub trait Instruction<M: Machine> {
    const OPCODE: u32;

    fn execute(state: &mut M, ops: Operands<Fp>);
}
