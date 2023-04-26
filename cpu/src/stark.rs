use crate::columns::CpuCols;
use core::borrow::Borrow;
use p3_air::constraint_consumer::ConstraintConsumer;
use p3_air::types::AirTypes;
use p3_air::window::AirWindow;
use p3_air::Air;
use p3_field::field::Field;
use p3_matrix::Matrix;

pub struct CpuStark;

impl<T, W, CC> Air<T, W, CC> for CpuStark
where
    T: AirTypes,
    W: AirWindow<T::Var>,
    CC: ConstraintConsumer<T>,
{
    fn eval(&self, window: &W, constraints: &mut CC) {
        let main = window.main();
        let local: &CpuCols<T::Var> = main.row(0).borrow();
        let next: &CpuCols<T::Var> = main.row(1).borrow();

        // TODO: Move to own function.
        let local_opcode_flags = &local.opcode_flags;
        let increment_pc = local_opcode_flags.is_imm32 + local_opcode_flags.is_bus_op;
        let transition = T::F::ONE; // TODO
        constraints
            .when(transition)
            .when(increment_pc)
            .assert_eq(next.pc, local.pc + T::F::ONE);
    }
}
