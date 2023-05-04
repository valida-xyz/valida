use crate::columns::CpuCols;
use core::borrow::Borrow;
use p3_air::{Air, AirBuilder};
use p3_field::field::FieldLike;
use p3_matrix::Matrix;

pub struct CpuStark;

impl<AB: AirBuilder> Air<AB> for CpuStark {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &CpuCols<AB::Var> = main.row(0).borrow();
        let next: &CpuCols<AB::Var> = main.row(1).borrow();

        self.eval_pc(builder, local, next);
        self.eval_fp(builder, local, next);
        self.eval_equality(builder, local, next);
    }
}

impl CpuStark {
    fn eval_pc<AB: AirBuilder>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
    ) {
        let should_increment_pc = local.opcode_flags.is_imm32 + local.opcode_flags.is_bus_op;
        let incremented_pc = local.pc + AB::F::ONE;
        builder
            .when_transition()
            .when(should_increment_pc)
            .assert_eq(next.pc, incremented_pc.clone());

        // Check if the first two operands are equal, in case we're doing a conditional branch.
        // TODO: Code below assumes that they're coming from memory, not immediates.
        builder.assert_eq(
            local.diff,
            local
                .mem_read_1()
                .0
                .into_iter()
                .zip(next.mem_read_2().0)
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<AB::FL>(),
        );
        builder.assert_bool(local.not_equal);
        builder.assert_eq(local.not_equal, local.diff * local.diff_inv);

        // Branch manipulation
        let equal = AB::FL::from(AB::F::ONE) - local.not_equal;
        let next_pc_if_branching = local.pc + local.instruction.operands.a();
        let beq_next_pc =
            equal.clone() * next_pc_if_branching.clone() + local.not_equal * incremented_pc.clone();
        let bne_next_pc = equal * incremented_pc + local.not_equal * next_pc_if_branching;
        builder
            .when_transition()
            .when(local.opcode_flags.is_beq)
            .assert_eq(next.pc, beq_next_pc);
        builder
            .when_transition()
            .when(local.opcode_flags.is_bne)
            .assert_eq(next.pc, bne_next_pc);

        // Jump manipulation
        builder
            .when_transition()
            .when(local.opcode_flags.is_jal)
            .assert_eq(next.pc, local.instruction.operands.b());
        builder
            .when_transition()
            .when(local.opcode_flags.is_jalv)
            .assert_eq(next.pc, local.mem_read_1()[3]);
    }

    fn eval_fp<AB: AirBuilder>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
    ) {
        builder
            .when_transition()
            .when(local.opcode_flags.is_jal)
            .assert_eq(next.fp, local.instruction.operands.c());

        builder
            .when_transition()
            .when(local.opcode_flags.is_jalv)
            .assert_eq(next.fp, local.mem_channels[0].value[3]);
    }

    fn eval_equality<AB: AirBuilder>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
    ) {
        // Word equality constraints (for branch instructions)
        builder
            .when(local.instruction.operands.is_imm())
            .assert_eq(
                local.diff,
                local
                    .mem_read_1()
                    .into_iter()
                    .zip(local.mem_read_2())
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<AB::FL>(),
            );
        builder
            .when(AB::FL::from(AB::F::ONE) - local.instruction.operands.is_imm())
            .assert_eq(
                local.diff,
                local.mem_read_1()[3] - local.instruction.operands.c(),
            );
        builder.assert_bool(local.not_equal);
        builder.assert_eq(local.not_equal, local.diff * local.diff_inv);
    }
}
