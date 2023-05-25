extern crate alloc;

use alloc::vec::Vec;
use columns::{Mul32Cols, NUM_MUL_COLS};
use core::marker::Sync;
use core::mem::transmute;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};

use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Mul32(Word<u8>, Word<u8>, Word<u8>),
}

#[derive(Default)]
pub struct Mul32Chip {
    pub clock: u32,
    pub operations: Vec<Operation>,
}

impl<M> Chip<M> for Mul32Chip
where
    M: MachineWithMul32Chip + Sync,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .cloned()
            .map(|op| self.op_to_row(op, machine))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_MUL_COLS)
    }

    fn local_sends(&self) -> Vec<Interaction<M::F>> {
        todo!()
    }
}

impl Mul32Chip {
    fn op_to_row<F, M>(&self, op: Operation, _machine: &M) -> [F; NUM_MUL_COLS]
    where
        F: PrimeField,
        M: MachineWithMul32Chip<F = F>,
    {
        let mut row = [F::ZERO; NUM_MUL_COLS];
        let mut cols: &mut Mul32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Mul32(a, b, c) => {
                cols.input_1 = b.to_field();
                cols.input_2 = c.to_field();
                cols.output = a.to_field();
            }
        }
        row
    }
}

pub trait MachineWithMul32Chip: MachineWithCpuChip {
    fn mul_u32(&self) -> &Mul32Chip;
    fn mul_u32_mut(&mut self) -> &mut Mul32Chip;
}

instructions!(Mul32Instruction);

impl<M> Instruction<M> for Mul32Instruction
where
    M: MachineWithMul32Chip,
{
    const OPCODE: u32 = 10;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c: Word<u8> = if ops.is_imm() == 1 {
            (ops.c() as u32).into()
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true).into()
        };

        let a = b * c;
        state.mem_mut().write(clk, write_addr, a.into(), true);

        state
            .mul_u32_mut()
            .operations
            .push(Operation::Mul32(a, b, c));
        state.cpu_mut().clock += 1;
        state.cpu_mut().pc += 1;
    }
}
