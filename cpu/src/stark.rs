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
    AB: AirBuilder<F = F>,
    F: PrimeField,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &CpuCols<AB::Var> = main.row(0).borrow();
        let next: &CpuCols<AB::Var> = main.row(1).borrow();

        self.eval_pc(builder, local, next);
        self.eval_fp(builder, local, next);
        self.eval_equality(builder, local, next);
        self.eval_memory_channels(builder, local, next);
    }
}

impl CpuStark {
    fn eval_memory_channels<F, AB>(
        &self,
        builder: &mut AB,
        local: &CpuCols<AB::Var>,
        next: &CpuCols<AB::Var>,
    ) where
        AB: AirBuilder<F = F>,
        F: PrimeField,
    {
        let is_load = local.opcode_flags.is_load;
        let is_store = local.opcode_flags.is_store;

        let mut base: [AB::Exp; 4] = unsafe { MaybeUninit::uninit().assume_init() };
        for (i, b) in [1 << 24, 1 << 16, 1 << 8, 1].into_iter().enumerate() {
            base[i] = AB::Exp::from(AB::F::from_canonical_u32(b));
        }

        // Read (1)
        let read_addr_1 = local.fp + local.instruction.operands.c();
        builder
            .when(is_load.clone() + is_store.clone())
            .assert_eq(local.mem_channels[0].addr, read_addr_1);
        builder
            .when(is_load.clone() + is_store.clone())
            .assert_one(local.mem_channels[0].used);
        builder
            .when(is_load.clone() + is_store.clone())
            .assert_one(local.mem_channels[0].is_read);

        // Read (2)
        let read_addr_2: AB::Exp = sigma::<F, AB>(&base, local.mem_channels[0].value);
        builder
            .when(is_load.clone())
            .assert_eq(local.mem_channels[1].addr, read_addr_2);
        builder
            .when(is_load.clone())
            .assert_one(local.mem_channels[1].used);
        builder
            .when(is_load.clone())
            .assert_one(local.mem_channels[1].is_read);

        // Write
        let write_addr_load = local.fp + local.instruction.operands.a();
        let write_addr_store = local.fp + local.instruction.operands.b();
        builder
            .when(is_load.clone())
            .assert_eq(local.mem_channels[2].addr, write_addr_load);
        builder
            .when(is_store.clone())
            .assert_eq(local.mem_channels[2].addr, write_addr_store);
        builder
            .when(is_store.clone() + is_load.clone())
            .assert_one(local.mem_channels[2].used);
        builder
            .when(is_store.clone() + is_load.clone())
            .assert_zero(local.mem_channels[2].is_read);
    }

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
                .sum::<AB::Exp>(),
        );
        builder.assert_bool(local.not_equal);
        builder.assert_eq(local.not_equal, local.diff * local.diff_inv);

        // Branch manipulation
        let equal = AB::Exp::from(AB::F::ONE) - local.not_equal;
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
        builder.when(local.instruction.operands.is_imm()).assert_eq(
            local.diff,
            local
                .mem_read_1()
                .into_iter()
                .zip(local.mem_read_2())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<AB::Exp>(),
        );
        builder
            .when(AB::Exp::from(AB::F::ONE) - local.instruction.operands.is_imm())
            .assert_eq(
                local.diff,
                local.mem_read_1()[3] - local.instruction.operands.c(),
            );
        builder.assert_bool(local.not_equal);
        builder.assert_eq(local.not_equal, local.diff * local.diff_inv);
    }
}

fn sigma<F: PrimeField, AB: AirBuilder<F = F>>(base: &[AB::Exp], input: Word<AB::Var>) -> AB::Exp {
    input
        .into_iter()
        .enumerate()
        .map(|(i, x)| base[i].clone() * x)
        .sum()
}
