use crate::columns::CpuCols;
use core::borrow::Borrow;
use p3_air::constraint_consumer::ConstraintConsumer;
use p3_air::types::AirTypes;
use p3_air::window::AirWindow;
use p3_air::Air;
use p3_field::field::Field;
use p3_matrix::Matrix;

pub struct CpuStark;

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
        let local: &CpuCols<T::Var> = main.row(0).borrow();
        let next: &CpuCols<T::Var> = main.row(1).borrow();

        self.eval_pc(constraints, local, next);
    }
}

impl CpuStark {
    fn eval_pc<T, W, CC>(
        &self,
        constraints: &mut CC,
        local: &CpuCols<T::Var>,
        next: &CpuCols<T::Var>,
    ) where
        T: AirTypes,
        W: AirWindow<T>,
        CC: ConstraintConsumer<T, W>,
    {
        let should_increment_pc = local.opcode_flags.is_imm32 + local.opcode_flags.is_bus_op;
        let incremented_pc = local.pc + T::F::ONE;
        constraints
            .when_transition()
            .when(should_increment_pc)
            .assert_eq(next.pc, incremented_pc.clone());

        constraints.assert_eq(
            local.diff,
            local
                .mem_read_1
                .0
                .into_iter()
                .zip(next.mem_read_1.0)
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<T::Exp>(),
        );

        constraints.assert_bool(local.not_equal);
        constraints.assert_eq(local.not_equal, local.diff * local.diff_inv);
        let equal = T::Exp::from(T::F::ONE) - local.not_equal.clone();

        // TODO: Should be the immediate jump destination or another read?
        let beq_next_pc_if_branching = incremented_pc.clone();

        let beq_next_pc = equal * beq_next_pc_if_branching + local.not_equal * incremented_pc;

        constraints
            .when_transition()
            .when(local.opcode_flags.is_beq)
            .assert_eq(next.pc, beq_next_pc);
    }
}
