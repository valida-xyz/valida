use crate::columns::MemoryCols;
use crate::MemoryChip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for MemoryChip {}

impl<AB> Air<AB> for MemoryChip
where
    AB: AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        self.eval_main(builder);
    }
}

impl MemoryChip {
    fn eval_main<AB: AirBuilder>(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &MemoryCols<AB::Var> = main.row_slice(0).borrow();
        let next: &MemoryCols<AB::Var> = main.row_slice(1).borrow();

        // Address equality
        builder
            .when_transition()
            .when(local.addr_not_equal)
            .assert_one((next.addr - local.addr) * local.diff_inv);
        builder.assert_bool(local.addr_not_equal);

        // Non-contiguous
        builder
            .when_transition()
            .when(local.addr_not_equal)
            .assert_eq(local.diff, next.addr - local.addr);
        builder
            .when_transition()
            .when_ne(local.addr_not_equal, AB::Expr::ONE)
            .assert_eq(local.diff, next.clk - local.clk);

        // Read/write
        // TODO: Record \sum_i (value'_i - value_i)^2 in trace and convert to a single constraint?
        for (value_next, value) in next.value.into_iter().zip(local.value.into_iter()) {
            builder
                .when_transition()
                .when(next.is_read)
                .when(next.is_real) // FIXME: Degree constraint 4, need to remove
                .when_ne(local.addr_not_equal, AB::Expr::ONE)
                .assert_eq(value_next, value);
        }
        builder
            .when(next.is_read)
            .when(next.is_real)
            .assert_eq(local.addr, next.addr);

        // Counter increments from zero.
        builder.when_first_row().assert_zero(local.counter);
        builder
            .when_transition()
            .assert_eq(next.counter, local.counter + AB::Expr::ONE);
    }
}
