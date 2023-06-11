use crate::columns::MemoryCols;
use crate::{MachineWithMemBus, MachineWithMemoryChip, MemoryChip};
use core::borrow::Borrow;
use p3_air::{Air, AirBuilder, PermutationAirBuilder};
use p3_field::{AbstractExtensionField, AbstractField, PrimeField};
use p3_matrix::Matrix;
use valida_machine::{chip, Machine};

impl<F, EF, AB, M> Air<AB> for MemoryChip<M>
where
    F: PrimeField,
    EF: AbstractExtensionField<AB::Expr> + From<AB::Expr> + Sync,
    AB: PermutationAirBuilder<F = F, EF = EF>,
    M: MachineWithMemoryChip<F = F, EF = EF> + MachineWithMemBus,
{
    fn eval(&self, builder: &mut AB) {
        self.eval_main(builder);
        chip::eval_permutation_constraints::<F, AB, EF, M, Self>(self, builder);
    }
}

impl<F, M> MemoryChip<M>
where
    F: PrimeField,
    M: MachineWithMemoryChip<F = F> + MachineWithMemBus,
{
    fn eval_main<AB: PermutationAirBuilder<F = F>>(&self, builder: &mut AB) {
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
