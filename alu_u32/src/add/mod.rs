extern crate alloc;

use alloc::vec::Vec;
use columns::{Add32Cols, NUM_ADD_COLS};
use core::mem::transmute;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{
    instructions, Chip, Instruction, Interaction, Operands, Word, MEMORY_CELL_BYTES,
};

use p3_field::{Field, PrimeField, PrimeField32, PrimeField64};
use p3_matrix::dense::RowMajorMatrix;

pub mod columns;
mod stark;

#[derive(Clone)]
pub enum Operation<F> {
    Add32(Word<F>, Word<F>, Word<F>),
}

#[derive(Default)]
pub struct Add32Chip<F> {
    pub clock: F,
    pub operations: Vec<Operation<F>>,
}

impl<M> Chip<M> for Add32Chip<M::F>
where
    M: MachineWithAdd32Chip,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .iter()
            .cloned()
            .map(|op| self.op_to_row(op, machine))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_ADD_COLS)
    }

    fn global_receives(&self, _machine: &M) -> Vec<Interaction<M::F>> {
        todo!()
    }

    fn global_sends(&self, _machine: &M) -> Vec<Interaction<M::F>> {
        todo!()
    }
}

impl<F: Field> Add32Chip<F> {
    fn op_to_row<M>(&self, op: Operation<F>, _machine: &M) -> [F; NUM_ADD_COLS]
    where
        M: MachineWithAdd32Chip<F = F>,
    {
        let mut row = [F::ZERO; NUM_ADD_COLS];
        let mut cols: &mut Add32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Add32(a, b, c) => {
                cols.input_1 = b;
                cols.input_2 = c;
                cols.output = a;
            }
        }
        row
    }
}

pub trait MachineWithAdd32Chip: MachineWithCpuChip {
    fn add_u32(&self) -> &Add32Chip<Self::F>;
    fn add_u32_mut(&mut self) -> &mut Add32Chip<Self::F>;
}

instructions!(Add32Instruction);

impl<F, M> Instruction<M> for Add32Instruction
where
    F: PrimeField32,
    M: MachineWithAdd32Chip<F = F>,
{
    const OPCODE: u32 = 8;

    fn execute(state: &mut M, ops: Operands<F>) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.b();
        let write_addr = state.cpu().fp + ops.a();
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c = if ops.is_imm() == F::ONE {
            ops.c().into()
        } else {
            let read_addr_2 = state.cpu().fp + ops.c();
            state.mem_mut().read(clk, read_addr_2, true)
        };

        let a = b + c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .add_u32_mut()
            .operations
            .push(Operation::Add32(a, b, c));
        state.cpu_mut().clock += F::ONE;
        state.cpu_mut().pc += F::ONE;
        // TODO: Set register log in the CPU as well
    }
}
