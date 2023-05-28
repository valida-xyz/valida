use crate::columns::CpuCols;
use crate::Store32Instruction;
use core::borrow::Borrow;
use core::mem::MaybeUninit;
use valida_machine::{Instruction, Word};

use p3_air::{Air, AirBuilder};
use p3_field::{AbstractField, PrimeField};
use p3_matrix::Matrix;

#[derive(Default)]
pub struct CpuStark;

impl<F, AB> Air<AB> for CpuStark
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &CpuCols<AB::Var> = main.row(0).borrow();
        let next: &CpuCols<AB::Var> = main.row(1).borrow();

        let base = [1 << 24, 1 << 16, 1 << 8, 1 << 0]
            .map(|b| AB::Expr::from(AB::F::from_canonical_u32(b)));

        self.eval_pc(builder, local, next, &base);
        self.eval_fp(builder, local, next, &base);
        self.eval_equality(builder, local, next, &base);
        self.eval_memory_channels(builder, local, next, &base);

        // Clock constraints
        builder.when_first_row().assert_zero(local.clk);
        builder
            .when_transition()
            .assert_eq(local.clk + AB::Expr::from(AB::F::ONE), next.clk);
    }
}

impl CpuStark {
    fn eval_memory_channels<F, AB>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
        base: &[AB::Expr; 4],
    ) where
        F: PrimeField,
        AB: AirBuilder<F = F>,
    {
        let is_load = local.opcode_flags.is_load;
        let is_store = local.opcode_flags.is_store;
        let is_jal = local.opcode_flags.is_jal;
        let is_jalv = local.opcode_flags.is_jalv;
        let is_beq = local.opcode_flags.is_beq;
        let is_bne = local.opcode_flags.is_bne;
        let is_imm32 = local.opcode_flags.is_imm32;
        let is_imm_op = local.opcode_flags.is_imm_op;

        let addr_a = local.fp + local.instruction.operands.a();
        let addr_b = local.fp + local.instruction.operands.b();
        let addr_c = local.fp + local.instruction.operands.c();

        builder.assert_one(local.mem_channels[0].is_read);
        builder.assert_one(local.mem_channels[1].is_read);
        builder.assert_zero(local.mem_channels[2].is_read);

        // Read (1)
        builder
            .when(is_load + is_store)
            .assert_eq(local.read_addr_1(), addr_c.clone());
        builder
            .when(is_jalv + is_beq + is_bne)
            .assert_eq(local.read_addr_1(), addr_b.clone());
        builder
            .when(is_load + is_store + is_jalv + is_beq + is_bne)
            .assert_one(local.read_1_used());

        // Read (2)
        builder.when(is_load).assert_eq(
            local.read_addr_2(),
            reduce::<F, AB>(base, local.read_value_1()),
        );
        builder
            .when(is_jalv + is_imm_op * (is_beq + is_bne))
            .assert_eq(local.read_addr_2(), addr_c);
        builder
            .when(is_store + is_load + is_jalv + is_beq + is_bne)
            .assert_one(local.read_2_used());

        // Write
        builder
            .when(is_load + is_jal + is_jalv + is_imm32)
            .assert_eq(local.write_addr(), addr_a);
        builder.when(is_store).assert_eq(local.write_addr(), addr_b);
        builder.when(is_load + is_store).assert_eq(
            reduce::<F, AB>(base, local.read_value_2()),
            reduce::<F, AB>(base, local.write_value()),
        );
        builder
            .when(is_jal + is_jalv)
            .assert_eq(reduce::<F, AB>(base, local.write_value()), next.pc);
        builder.when(is_imm32).assert_eq(
            // For all imm32 instructions, program memory is trusted to have operand values
            // between 0 and 255.
            reduce::<F, AB>(base, local.write_value()),
            reduce::<F, AB>(
                base,
                Word([
                    local.instruction.operands.0[0],
                    local.instruction.operands.0[1],
                    local.instruction.operands.0[2],
                    local.instruction.operands.0[3],
                ]),
            ),
        );
        builder
            .when(is_store + is_load + is_jal + is_jalv + is_imm32)
            .assert_one(local.write_used());
    }

    fn eval_pc<F, AB>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
        base: &[AB::Expr; 4],
    ) where
        F: PrimeField,
        AB: AirBuilder<F = F>,
    {
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
                .read_value_1()
                .0
                .into_iter()
                .zip(next.read_value_2().0)
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<AB::Expr>(),
        );
        builder.assert_bool(local.not_equal);
        builder.assert_eq(local.not_equal, local.diff * local.diff_inv);

        // Branch manipulation
        let equal = AB::Expr::from(AB::F::ONE) - local.not_equal;
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
            .assert_eq(next.pc, reduce::<F, AB>(base, local.read_value_1()));
    }

    fn eval_fp<F, AB>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
        base: &[AB::Expr; 4],
    ) where
        F: PrimeField,
        AB: AirBuilder<F = F>,
    {
        builder
            .when_transition()
            .when(local.opcode_flags.is_jal)
            .assert_eq(next.fp, local.fp + local.instruction.operands.c());

        builder
            .when_transition()
            .when(local.opcode_flags.is_jalv)
            .assert_eq(next.fp, reduce::<F, AB>(base, local.read_value_2()));
    }

    fn eval_equality<AB: AirBuilder>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        _next: &CpuCols<AB::Var>,
        _base: &[AB::Expr; 4],
    ) {
        // Word equality constraints (for branch instructions)
        builder.when(local.instruction.operands.is_imm()).assert_eq(
            local.diff,
            local
                .read_value_1()
                .into_iter()
                .zip(local.read_value_2())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<AB::Expr>(),
        );
        builder
            .when(AB::Expr::from(AB::F::ONE) - local.instruction.operands.is_imm())
            .assert_eq(
                local.diff,
                local.read_value_1()[3] - local.instruction.operands.c(),
            );
        builder.assert_bool(local.not_equal);
        builder.assert_eq(local.not_equal, local.diff * local.diff_inv);
    }
}

fn reduce<F: PrimeField, AB: AirBuilder<F = F>>(
    base: &[AB::Expr],
    input: Word<AB::Var>,
) -> AB::Expr {
    input
        .into_iter()
        .enumerate()
        .map(|(i, x)| base[i].clone() * x)
        .sum()
}
