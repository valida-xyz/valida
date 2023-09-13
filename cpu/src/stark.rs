use crate::columns::CpuCols;
use crate::CpuChip;
use core::borrow::Borrow;
use valida_machine::Word;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField};
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for CpuChip {}

impl<F, AB> Air<AB> for CpuChip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &CpuCols<AB::Var> = main.row_slice(0).borrow();
        let next: &CpuCols<AB::Var> = main.row_slice(1).borrow();

        let base = [1 << 24, 1 << 16, 1 << 8, 1].map(AB::Expr::from_canonical_u32);

        self.eval_pc(builder, local, next, &base);
        self.eval_fp(builder, local, next, &base);
        self.eval_equality(builder, local, next, &base);
        self.eval_memory_channels(builder, local, next, &base);

        // Clock constraints
        builder.when_first_row().assert_zero(local.clk);
        builder
            .when_transition()
            .assert_eq(local.clk + AB::Expr::ONE, next.clk);
        builder
            .when(local.opcode_flags.is_bus_op_with_mem)
            .assert_eq(local.clk, local.chip_channel.clk_or_zero);
        builder
            .when(AB::Expr::ONE - local.opcode_flags.is_bus_op_with_mem)
            .assert_zero(local.chip_channel.clk_or_zero);

        // Immediate value constraints (TODO: we'd need to range check read_value_2 in
        // this case)
        builder.when(local.opcode_flags.is_imm_op).assert_eq(
            local.instruction.operands.c(),
            reduce::<AB>(&base, local.read_value_2()),
        );

        // "Stop" constraints (to check that program execution was not stopped prematurely)
        builder
            .when_transition()
            .when(local.opcode_flags.is_stop)
            .assert_eq(next.pc, local.pc);
        builder
            .when_last_row()
            .assert_one(local.opcode_flags.is_stop);
    }
}

impl CpuChip {
    fn eval_memory_channels<AB>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        _next: &CpuCols<AB::Var>, // TODO: unused
        base: &[AB::Expr; 4],
    ) where
        AB: AirBuilder,
    {
        let is_load = local.opcode_flags.is_load;
        let is_store = local.opcode_flags.is_store;
        let is_jal = local.opcode_flags.is_jal;
        let is_jalv = local.opcode_flags.is_jalv;
        let is_beq = local.opcode_flags.is_beq;
        let is_bne = local.opcode_flags.is_bne;
        let is_imm32 = local.opcode_flags.is_imm32;
        let _is_advice = local.opcode_flags.is_advice; // TODO: unused
        let is_imm_op = local.opcode_flags.is_imm_op;
        let is_bus_op = local.opcode_flags.is_bus_op;
        let _is_bus_op_with_mem = local.opcode_flags.is_bus_op_with_mem; // TODO: unused

        let addr_a = local.fp + local.instruction.operands.a();
        let addr_b = local.fp + local.instruction.operands.b();
        let addr_c = local.fp + local.instruction.operands.c();

        builder.assert_one(local.mem_channels[0].is_read);
        builder.assert_one(local.mem_channels[1].is_read);
        builder.assert_zero(local.mem_channels[2].is_read);

        // Read (1)
        builder
            .when(is_jalv + is_beq + is_bne + is_bus_op)
            .assert_eq(local.read_addr_1(), addr_b.clone());
        builder
            .when(is_load + is_store)
            .assert_eq(local.read_addr_1(), addr_c.clone());
        builder
            .when(is_load + is_store + is_jalv + is_beq + is_bne + is_bus_op)
            .assert_one(local.read_1_used());
        builder.when(is_jal).assert_zero(local.read_1_used());

        // Read (2)
        builder.when(is_load).assert_eq(
            local.read_addr_2(),
            reduce::<AB>(base, local.read_value_1()),
        );
        builder
            .when(is_jalv + (AB::Expr::ONE - is_imm_op) * (is_beq + is_bne + is_bus_op))
            .assert_eq(local.read_addr_2(), addr_c);
        builder
            .when(is_load + is_jalv + (AB::Expr::ONE - is_imm_op) * (is_beq + is_bne + is_bus_op))
            .assert_one(local.read_2_used());
        builder
            .when(is_store + is_jal + is_imm_op * (is_beq + is_bne + is_bus_op))
            .assert_zero(local.read_2_used());

        // Write
        builder
            .when(is_load + is_jal + is_jalv + is_imm32 + is_bus_op)
            .assert_eq(local.write_addr(), addr_a);
        builder.when(is_store).assert_eq(local.write_addr(), addr_b);
        builder.when(is_store).assert_zero(
            local
                .read_value_1()
                .into_iter()
                .zip(local.write_value())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<AB::Expr>(),
        );
        builder.when(is_load).assert_zero(
            local
                .read_value_2()
                .into_iter()
                .zip(local.write_value())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<AB::Expr>(),
        );
        builder.when_transition().when(is_jal + is_jalv).assert_eq(
            local.pc + AB::F::ONE,
            reduce::<AB>(base, local.write_value()),
        );
        builder.when(is_imm32).assert_zero(
            local
                .write_value()
                .into_iter()
                .zip(local.instruction.operands.imm32())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<AB::Expr>(),
        );
        builder
            .when(is_store + is_load + is_jal + is_jalv + is_imm32 + is_bus_op)
            .assert_one(local.write_used());
    }

    fn eval_pc<AB>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
        base: &[AB::Expr; 4],
    ) where
        AB: AirBuilder,
    {
        let should_increment_pc = local.opcode_flags.is_imm32
            + local.opcode_flags.is_bus_op
            + local.opcode_flags.is_advice;
        let incremented_pc = local.pc + AB::F::ONE;
        builder
            .when_transition()
            .when(should_increment_pc)
            .assert_eq(next.pc, incremented_pc.clone());

        // Branch manipulation
        let equal = AB::Expr::ONE - local.not_equal;
        let next_pc_if_branching = local.instruction.operands.a();
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
            .assert_eq(next.pc, reduce::<AB>(base, local.read_value_1()));
    }

    fn eval_fp<AB>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
        base: &[AB::Expr; 4],
    ) where
        AB: AirBuilder,
    {
        builder
            .when_transition()
            .when(local.opcode_flags.is_jal)
            .assert_eq(next.fp, local.fp + local.instruction.operands.c());
        builder
            .when_transition()
            .when(local.opcode_flags.is_jalv)
            .assert_eq(next.fp, local.fp + reduce::<AB>(base, local.read_value_2()));
        builder
            .when_transition()
            .when(AB::Expr::ONE - local.opcode_flags.is_jal - local.opcode_flags.is_jalv)
            .assert_eq(next.fp, local.fp);
    }

    fn eval_equality<AB: AirBuilder>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        _next: &CpuCols<AB::Var>, // TODO: unused
        _base: &[AB::Expr; 4],    // TODO: unused
    ) {
        // Check if the first two operand values are equal, in case we're doing a conditional branch.
        // (when is_imm == 1, the second read value is guaranteed to be an immediate value)
        builder.assert_eq(
            local.diff,
            local
                .read_value_1()
                .into_iter()
                .zip(local.read_value_2())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<AB::Expr>(),
        );
        builder.assert_bool(local.not_equal);
        builder.assert_eq(local.not_equal, local.diff * local.diff_inv);
    }
}

fn reduce<AB: AirBuilder>(base: &[AB::Expr], input: Word<AB::Var>) -> AB::Expr {
    input
        .into_iter()
        .enumerate()
        .map(|(i, x)| base[i].clone() * x)
        .sum()
}
