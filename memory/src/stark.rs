use crate::columns::MemoryCols;
use crate::MemoryChip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder};
use p3_field::AbstractField;
use p3_matrix::MatrixRows;

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
        let local: &MemoryCols<AB::Var> = main.row(0).borrow();
        let next: &MemoryCols<AB::Var> = main.row(1).borrow();

        // Address equality
        builder.when_transition().assert_eq(
            local.addr_not_equal,
            (next.addr - local.addr) * next.diff_inv,
        );
        builder.assert_bool(local.addr_not_equal);

        // Non-contiguous
        builder
            .when_transition()
            .when(local.addr_not_equal)
            .assert_eq(next.diff, next.addr - local.addr);
        builder
            .when_transition()
            .when_ne(local.addr_not_equal, AB::Expr::from(AB::F::ONE))
            .assert_eq(next.diff, next.clk - local.clk - AB::Expr::from(AB::F::ONE));

        // Read/write
        // TODO: Record \sum_i (value'_i - value_i)^2 in trace and convert to a single constraint?
        for (value_next, value) in next.value.into_iter().zip(local.value.into_iter()) {
            let is_value_unchanged =
                (local.addr - next.addr + AB::Expr::from(AB::F::ONE)) * (value_next - value);
            builder
                .when_transition()
                .when(next.is_read)
                .assert_zero(is_value_unchanged);
        }

        // Counter
        builder
            .when_transition()
            .assert_eq(next.counter, local.counter + AB::Expr::from(AB::F::ONE));
    }
}
