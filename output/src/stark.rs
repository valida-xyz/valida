use crate::columns::{OutputCols, NUM_OUTPUT_COLS};
use crate::OutputChip;
use core::borrow::Borrow;
use valida_cpu::stark::reduce;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField};
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for OutputChip {
    fn width(&self) -> usize {
        NUM_OUTPUT_COLS
    }
}

impl<F, AB> Air<AB> for OutputChip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &OutputCols<AB::Var> = main.row_slice(0).borrow();
        let next: &OutputCols<AB::Var> = main.row_slice(1).borrow();

        let base = [1 << 24, 1 << 16, 1 << 8, 1].map(AB::Expr::from_canonical_u32);
        let diff = reduce::<AB>(&base, local.diff);
        builder
            .when_transition()
            .assert_eq(diff, next.clk - local.clk);
    }
}
