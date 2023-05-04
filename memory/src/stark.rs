use crate::columns::MemoryCols;
use core::borrow::Borrow;
use p3_air::constraint_consumer::ConstraintConsumer;
use p3_air::types::AirTypes;
use p3_air::window::AirWindow;
use p3_air::Air;
use p3_field::field::Field;
use p3_matrix::Matrix;

pub struct MemoryStark;

impl<T, W> Air<T, W> for MemoryStark
where
    T: AirTypes,
    W: AirWindow<T>,
{
    fn eval<CC>(&self, constraints: &mut CC)
    where
        CC: ConstraintConsumer<T, W>,
    {
        let main = constraints.window().main();
        let local: &MemoryCols<T::Var> = main.row(0).borrow();
        let next: &MemoryCols<T::Var> = main.row(1).borrow();

        // Address equality constraints
        constraints.when_transition().assert_eq(
            local.addr_not_equal,
            (next.addr - local.addr) * next.diff_inv,
        );
        constraints.assert_bool(local.addr_not_equal);

        // Non-contiguous
        constraints
            .when_transition()
            .when(local.addr_not_equal)
            .assert_eq(next.diff, next.addr - local.addr);
        constraints
            .when_transition()
            .when(T::Exp::from(T::F::ONE) - local.addr_not_equal)
            .assert_eq(next.diff, next.clk - local.clk - T::Exp::from(T::F::ONE));

        // Read/write
        // TODO: Record \sum_i (value'_i - value_i)^2 in trace and convert to a single constraint?
        for (value_next, value) in next.value.into_iter().zip(local.value.into_iter()) {
            let is_value_unchanged =
                (local.addr - next.addr + T::Exp::from(T::F::ONE)) * (value_next - value);
            constraints
                .when_transition()
                .when(next.is_read)
                .assert_zero(is_value_unchanged);
        }
    }
}
