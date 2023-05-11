use crate::columns::{MemoryCols, MemoryPermutationCols};
use core::borrow::Borrow;
use p3_air::{Air, AirBuilder, PermutationAirBuilder};
use p3_field::AbstractField;
use p3_matrix::Matrix;

pub struct MemoryStark;

impl<AB: PermutationAirBuilder> Air<AB> for MemoryStark {
    fn eval(&self, builder: &mut AB) {
        self.eval_main(builder);
        self.eval_permutation(builder);
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
            .when(AB::Exp::from(AB::F::ONE) - local.addr_not_equal)
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
    }

    // TODO: Refactor into range check library code. Use extension field variables
    // for randomness and permutation rows
    fn eval_permutation<AB: PermutationAirBuilder>(&self, builder: &mut AB) {
        let main = builder.main();
        let main_local: &MemoryCols<AB::Var> = main.row(0).borrow();
        let main_next: &MemoryCols<AB::Var> = main.row(1).borrow();

        let perm = builder.permutation();
        let perm_local: &MemoryPermutationCols<AB::Var> = perm.row(0).borrow();
        let perm_next: &MemoryPermutationCols<AB::Var> = perm.row(1).borrow();

        let rand_elems = builder.permutation_randomness();
        let rand0 = rand_elems[0].clone();
        let rand1 = rand_elems[1].clone();
        let rand2 = rand_elems[2].clone();

        // Plookup constraints
        builder
            .when_first_row()
            .assert_zero(perm_local.addr - perm_local.counter_addr);
        builder
            .when_first_row()
            .assert_zero(perm_local.diff - perm_local.counter_diff);
        builder.when_transition().assert_zero(
            (perm_local.addr - perm_local.counter_addr) * (perm_local.addr - perm_next.addr),
        );
        builder.when_transition().assert_zero(
            (perm_local.diff - perm_local.counter_diff) * (perm_local.diff - perm_next.diff),
        );

        // Permutation check
        builder
            .when_first_row()
            .assert_eq(perm_local.z, AB::Exp::from(AB::F::ONE));
        builder
            .when_last_row()
            .assert_eq(perm_local.z, AB::Exp::from(AB::F::ONE));
        builder.when_transition().assert_eq(
            perm_next.z
                * ((perm_local.addr + rand0.clone() * perm_local.diff) - rand1.clone())
                * ((perm_local.counter_addr + rand0.clone() * perm_local.counter_diff)
                    - rand2.clone()),
            perm_local.z
                * ((main_local.addr + rand0.clone() * main_local.diff) - rand1)
                * ((main_local.counter + rand0 * main_local.counter) - rand2),
        );
    }
}
