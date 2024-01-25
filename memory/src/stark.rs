use crate::columns::{MemoryCols, NUM_MEM_COLS};
use crate::MemoryChip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for MemoryChip {
    fn width(&self) -> usize {
        NUM_MEM_COLS
    }
}

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

        // Flags should be boolean.
        builder.assert_bool(local.is_read);
        builder.assert_bool(local.is_write);
        builder.assert_bool(local.is_read + local.is_write);
        builder.assert_bool(local.addr_not_equal);

        let addr_delta = next.addr - local.addr;
        let addr_equal = AB::Expr::one() - local.addr_not_equal;

        // Ensure addr_not_equal is set correctly.
        builder
            .when_transition()
            .when(local.addr_not_equal)
            .assert_one(addr_delta.clone() * local.diff_inv);
        builder
            .when_transition()
            .when(addr_equal.clone())
            .assert_zero(addr_delta.clone());

        // diff should match either the address delta or the clock delta, based on addr_not_equal.
        builder
            .when_transition()
            .when(local.addr_not_equal)
            .assert_eq(local.diff, addr_delta.clone());
        builder
            .when_transition()
            .when(addr_equal.clone())
            .assert_eq(local.diff, next.clk - local.clk);

        // Read/write
        // TODO: Record \sum_i (value'_i - value_i)^2 in trace and convert to a single constraint?
        for (value_next, value) in next.value.into_iter().zip(local.value.into_iter()) {
            builder
                .when_transition()
                .when(next.is_read)
                .when(addr_equal.clone())
                .assert_eq(value_next, value);
        }

        // TODO: This disallows reading unitialized memory? Not sure that's desired, it depends on
        // how we implement continuations. If we end up defaulting to zero, then we should replace
        // this with
        //     when(is_read).when(addr_delta).assert_zero(value_next);
        builder.when(next.is_read).assert_zero(addr_delta);

        // Counter increments from zero.
        builder.when_first_row().assert_zero(local.counter);
        builder
            .when_transition()
            .assert_eq(next.counter, local.counter + AB::Expr::one());
    }
}
