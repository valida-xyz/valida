extern crate alloc;

use alloc::vec::Vec;
use columns::{Add32Cols, NUM_ADD_COLS};
use core::mem::transmute;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};

use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;

pub mod columns;
mod stark;

#[derive(Clone)]
pub enum Operation {
    Add32(Word<u8>, Word<u8>, Word<u8>),
}

#[derive(Default)]
pub struct Add32Chip {
    pub clock: u32,
    pub operations: Vec<Operation>,
}

impl<M> Chip<M> for Add32Chip
where
    M: MachineWithAdd32Chip,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .iter()
            .map(|op| self.op_to_row::<M::F, M>(op))
            .flatten()
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows, NUM_ADD_COLS)
    }

    fn global_receives(&self, _machine: &M) -> Vec<Interaction<M::F>> {
        todo!()
    }

    fn global_sends(&self, _machine: &M) -> Vec<Interaction<M::F>> {
        todo!()
    }
}

impl Add32Chip {
    fn op_to_row<F, M>(&self, op: &Operation) -> [F; NUM_ADD_COLS]
    where
        F: PrimeField,
        M: MachineWithAdd32Chip<F = F>,
    {
        let mut row = [F::ZERO; NUM_ADD_COLS];
        let mut cols: &mut Add32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Add32(a, b, c) => {
                cols.input_1 = b.to_field();
                cols.input_2 = c.to_field();
                cols.output = a.to_field();
            }
        }
        row
    }
}

pub trait MachineWithAdd32Chip: MachineWithCpuChip {
    fn add_u32(&self) -> &Add32Chip;
    fn add_u32_mut(&mut self) -> &mut Add32Chip;
}

instructions!(Add32Instruction);

impl<M> Instruction<M> for Add32Instruction
where
    M: MachineWithAdd32Chip,
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

        let a = b + c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .add_u32_mut()
            .operations
            .push(Operation::Add32(a, b, c));
        state.cpu_mut().clock += 1;
        state.cpu_mut().pc += 1;
        // TODO: Set register log in the CPU as well
    }
}
