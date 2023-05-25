extern crate alloc;

use alloc::vec::Vec;
use columns::{Sub32Cols, NUM_SUB_COLS};
use core::mem::transmute;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Operands, Word};

use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Sub32(Word<u8>, Word<u8>, Word<u8>),
}

#[derive(Default)]
pub struct Sub32Chip {
    pub clock: u32,
    pub operations: Vec<Operation>,
}

impl<M> Chip<M> for Sub32Chip
where
    M: MachineWithSub32Chip,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row::<M::F, M>(op))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_SUB_COLS)
    }
}

impl Sub32Chip {
    fn op_to_row<F, M>(&self, op: &Operation) -> [F; NUM_SUB_COLS]
    where
        F: PrimeField,
        M: MachineWithSub32Chip<F = F>,
    {
        let mut row = [F::ZERO; NUM_SUB_COLS];
        let mut cols: &mut Sub32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Sub32(a, b, c) => {
                cols.input_1 = b.to_field();
                cols.input_2 = c.to_field();
                cols.output = a.to_field();
            }
        }
        row
    }
}

pub trait MachineWithSub32Chip: MachineWithCpuChip {
    fn sub_u32(&self) -> &Sub32Chip;
    fn sub_u32_mut(&mut self) -> &mut Sub32Chip;
}

instructions!(Sub32Instruction);

impl<M> Instruction<M> for Sub32Instruction
where
    M: MachineWithSub32Chip,
{
    const OPCODE: u32 = 8;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c = if ops.is_imm() == 1 {
            (ops.c() as u32).into()
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true)
        };

        let a = b - c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .sub_u32_mut()
            .operations
            .push(Operation::Sub32(a, b, c));
        state.cpu_mut().clock += 1;
        state.cpu_mut().pc += 1;
    }
}
