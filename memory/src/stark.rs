use crate::columns::MemoryCols;
use core::borrow::Borrow;
use p3_air::{Air, AirBuilder, PermutationAirBuilder};
use p3_field::AbstractField;
use p3_matrix::Matrix;

#[derive(Default)]
pub struct MemoryStark {}

impl<AB: PermutationAirBuilder> Air<AB> for MemoryStark {
    fn eval(&self, builder: &mut AB) {
        self.eval_main(builder);
    }
}

impl MemoryStark {
    fn eval_main<AB: PermutationAirBuilder>(&self, builder: &mut AB) {
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
            .when_ne(local.addr_not_equal, AB::Exp::from(AB::F::ONE))
            .assert_eq(next.diff, next.clk - local.clk - AB::Exp::from(AB::F::ONE));

        // Read/write
        // TODO: Record \sum_i (value'_i - value_i)^2 in trace and convert to a single constraint?
        for (value_next, value) in next.value.into_iter().zip(local.value.into_iter()) {
            let is_value_unchanged =
                (local.addr - next.addr + AB::Exp::from(AB::F::ONE)) * (value_next - value);
            builder
                .when_transition()
                .when(next.is_read)
                .assert_zero(is_value_unchanged);
        }

        // Counter
        builder
            .when_transition()
            .assert_eq(next.counter, local.counter + AB::Exp::from(AB::F::ONE));
    }
}
