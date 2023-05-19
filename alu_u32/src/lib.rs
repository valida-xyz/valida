#![no_std]

extern crate alloc;

use crate::columns::NUM_ALU_COLS;
use alloc::vec::Vec;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Instruction, Operands, Word, MEMORY_CELL_BYTES};

use p3_field::{AbstractField, PrimeField, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31 as Fp;
use valida_machine::Chip;

pub mod columns;
mod stark;

#[derive(Clone)]
pub enum Operation {
    Add32,
    Mul32,
}

#[derive(Default)]
pub struct ALU32Chip {
    pub clock: Fp,
    pub operations: Vec<Operation>,
}

impl<M> Chip<M> for ALU32Chip
where
    M: MachineWithALU32Chip,
{
    type F = Fp;
    type FE = Fp; // FIXME

    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<Self::F> {
        let rows = self
            .operations
            .iter()
            .cloned()
            .enumerate()
            .map(|(n, op)| self.op_to_row(op, machine))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_ALU_COLS)
    }

    fn generate_permutation_trace(
        &self,
        machine: &M,
        main_trace: RowMajorMatrix<Self::F>,
        random_elements: Vec<Self::FE>,
    ) -> RowMajorMatrix<Self::F> {
        todo!()
    }
}

impl ALU32Chip {
    fn op_to_row<M>(&self, op: Operation, _machine: &M) -> [Fp; NUM_ALU_COLS]
    where
        M: MachineWithALU32Chip,
    {
        todo!()
    }
}

pub trait MachineWithALU32Chip: MachineWithCpuChip {
    fn alu_u32(&self) -> &ALU32Chip;
    fn alu_u32_mut(&mut self) -> &mut ALU32Chip;
}

instructions!(Add32Instruction, Mul32Instruction);

impl<M: MachineWithALU32Chip> Instruction<M> for Add32Instruction {
    const OPCODE: u32 = 8;

    fn execute(state: &mut M, ops: Operands<Fp>) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.b();
        let write_addr = state.cpu().fp + ops.a();
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c = if ops.is_imm() == Fp::ONE {
            ops.c().into()
        } else {
            let read_addr_2 = state.cpu().fp + ops.c();
            state.mem_mut().read(clk, read_addr_2, true)
        };

        // FIXME
        let mut a = Word::<Fp>::default();
        let mut carry = 0u8;
        for i in (0..MEMORY_CELL_BYTES).rev() {
            let b_i = b[i].as_canonical_u32() as u8;
            let c_i = c[i].as_canonical_u32() as u8;
            let (sum, overflow) = b_i.overflowing_add(c_i);
            let (sum_with_carry, carry_overflow) = sum.overflowing_add(carry);
            carry = overflow as u8 + carry_overflow as u8;
            a[i] = Fp::from_canonical_u8(sum_with_carry);
        }
        state.mem_mut().write(clk, write_addr, a, true);

        state.alu_u32_mut().operations.push(Operation::Add32);
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().pc += Fp::ONE;
        // TODO: Set register log in the CPU as well
    }
}

impl<M: MachineWithALU32Chip> Instruction<M> for Mul32Instruction {
    const OPCODE: u32 = 9;

    fn execute(state: &mut M, ops: Operands<Fp>) {
        todo!()
    }
}
