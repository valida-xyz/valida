use crate::columns::{StaticDataCols, NUM_STATIC_DATA_COLS};
use crate::StaticDataChip;

use core::borrow::Borrow;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for StaticDataChip {
    fn width(&self) -> usize {
        NUM_STATIC_DATA_COLS
    }
}

impl<AB> Air<AB> for StaticDataChip
where
    AB: AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        self.eval_main(builder);
    }
}

impl StaticDataChip {
    fn eval_main<AB: AirBuilder>(&self, builder: &mut AB) {
        // ensure that addresses are sequentially increasing, in order to ensure internal consistency of static data trace
        let main = builder.main();
        let local: &StaticDataCols<AB::Var> = main.row_slice(0).borrow();
        let next: &StaticDataCols<AB::Var> = main.row_slice(1).borrow();
        builder
            .when_transition()
            .when(local.is_real * next.is_real)
            .assert_eq(
                next.addr,
                local.addr + AB::Expr::one() + AB::Expr::one() + AB::Expr::one() + AB::Expr::one(),
            );
    }
}
