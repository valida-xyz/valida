use crate::columns::{NUM_PROGRAM_COLS};
use crate::ProgramChip;
use alloc::vec;
use valida_machine::InstructionWord;

use p3_air::{Air, BaseAir, PairBuilder};
use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;

impl<AB> Air<AB> for ProgramChip
where
    AB: PairBuilder,
{
    fn eval(&self, _builder: &mut AB) {}
}

impl<F: Field> BaseAir<F> for ProgramChip {
    fn width(&self) -> usize {
        NUM_PROGRAM_COLS
    }
}
