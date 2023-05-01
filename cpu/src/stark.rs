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
        self.eval_fp(constraints, local, next);
        self.eval_equality(constraints, local, next);
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

        // Branch manipulation
        let equal = T::Exp::from(T::F::ONE) - local.not_equal.clone();
        let next_pc_if_branching = local.pc + local.instruction.operands.a();
        let beq_next_pc =
            equal.clone() * next_pc_if_branching.clone() + local.not_equal * incremented_pc.clone();
        let bne_next_pc = equal * incremented_pc + local.not_equal * next_pc_if_branching;
        constraints
            .when_transition()
            .when(local.opcode_flags.is_beq)
            .assert_eq(next.pc, beq_next_pc);
        constraints
            .when_transition()
            .when(local.opcode_flags.is_bne)
            .assert_eq(next.pc, bne_next_pc);

        // Jump manipulation
        constraints
            .when_transition()
            .when(local.opcode_flags.is_jal)
            .assert_eq(next.pc, local.instruction.operands.b());
        constraints
            .when_transition()
            .when(local.opcode_flags.is_jalv)
            .assert_eq(next.pc, local.mem_read_1()[3]);
    }

    fn eval_fp<T, W, CC>(
        &self,
        constraints: &mut CC,
        local: &CpuCols<T::Var>,
        next: &CpuCols<T::Var>,
    ) where
        T: AirTypes,
        W: AirWindow<T>,
        CC: ConstraintConsumer<T, W>,
    {
        constraints
            .when_transition()
            .when(local.opcode_flags.is_jal)
            .assert_eq(next.fp, local.instruction.operands.c());

        constraints
            .when_transition()
            .when(local.opcode_flags.is_jalv)
            .assert_eq(next.fp, local.mem_channels[0].value[3]);
    }

    fn eval_equality<T, W, CC>(
        &self,
        constraints: &mut CC,
        local: &CpuCols<T::Var>,
        next: &CpuCols<T::Var>,
    ) where
        T: AirTypes,
        W: AirWindow<T>,
        CC: ConstraintConsumer<T, W>,
    {
        // Word equality constraints (for branch instructions)
        constraints
            .when(local.instruction.operands.is_imm())
            .assert_eq(
                local.diff,
                local
                    .mem_read_1()
                    .into_iter()
                    .zip(local.mem_read_2())
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<T::Exp>(),
            );
        constraints
            .when(T::Exp::from(T::F::ONE) - local.instruction.operands.is_imm())
            .assert_eq(
                local.diff,
                local.mem_read_1()[3] - local.instruction.operands.c(),
            );
        constraints.assert_bool(local.not_equal);
        constraints.assert_eq(local.not_equal, local.diff * local.diff_inv);
    }
}
