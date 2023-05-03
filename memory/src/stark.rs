use crate::columns::MemoryCols;
use core::borrow::Borrow;
use p3_air::constraint_consumer::ConstraintConsumer;
use p3_air::types::AirTypes;
use p3_air::window::AirWindow;
use p3_air::Air;
use p3_field::field::Field;
use p3_matrix::Matrix;

pub struct MemoryStark;

impl<T, W> Air<T, W> for CpuStark
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

        let is_value_unchanged =
            (next.address - local.address + T::Exp::from(T::F::ONE)) * (next.value - local.value);
        constraints
            .when_transition()
            .when(next.is_read)
            .assert_zero(is_value_unchanged);

        constraints
            .when_transition()
            .assert_eq(local.diff, next.addr - local.addr)
    }
}
