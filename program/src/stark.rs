use crate::columns::{ProgramCols, NUM_PREPROCESSED_COLS};
use crate::ProgramChip;
use core::borrow::Borrow;
use valida_machine::InstructionWord;

use p3_air::{Air, AirBuilder, BaseAir, PairBuilder};
use p3_field::{AbstractField, PrimeField32};
use p3_matrix::{dense::RowMajorMatrix, MatrixRowSlices};

impl<F, AB> Air<AB> for ProgramChip
where
    F: PrimeField32,
    AB: PairBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &ProgramCols<AB::Var> = main.row_slice(0).borrow();
        let next: &ProgramCols<AB::Var> = main.row_slice(1).borrow();

        builder.when_first_row().assert_zero(local.counter);
        builder
            .when_transition()
            .when_ne(next.multiplicity, AB::Expr::ZERO)
            .assert_eq(next.counter, local.counter + AB::Expr::ONE);
    }
}

impl<F: PrimeField32> BaseAir<F> for ProgramChip {
    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        // Pad the ROM to a power of two.
        let mut rom = self.program_rom.0.clone();
        let n = rom.len();
        rom.resize(n.next_power_of_two(), InstructionWord::default());

        let flattened = rom.into_iter().flat_map(|word| word.flatten()).collect();
        let trace = RowMajorMatrix::new(flattened, NUM_PREPROCESSED_COLS);
        Some(trace)
    }
}
