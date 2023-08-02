extern crate alloc;

use crate::{pad_to_power_of_two, DIV32_OPCODE};
use alloc::vec::Vec;
use columns::NUM_DIV_COLS;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Operands, Word};
use valida_range::MachineWithRangeChip;

use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Div32(Word<u8>, Word<u8>, Word<u8>), // (quotient, dividend, divisor)
}

#[derive(Default)]
pub struct Div32Chip {
    pub operations: Vec<Operation>,
}

impl<F, M> Chip<M> for Div32Chip
where
    F: PrimeField,
    M: MachineWithGeneralBus<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();

        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_DIV_COLS);

        pad_to_power_of_two::<NUM_DIV_COLS, F>(&mut trace.values);

        trace
    }
}

impl Div32Chip {
    fn op_to_row<F>(&self, _op: &Operation) -> [F; NUM_DIV_COLS]
    where
        F: PrimeField,
    {
        [F::ZERO; NUM_DIV_COLS]
    }
}

pub trait MachineWithDiv32Chip: MachineWithCpuChip {
    fn div_u32(&self) -> &Div32Chip;
    fn div_u32_mut(&mut self) -> &mut Div32Chip;
}

instructions!(Div32Instruction);

impl<M> Instruction<M> for Div32Instruction
where
    M: MachineWithDiv32Chip + MachineWithRangeChip,
{
    const OPCODE: u32 = DIV32_OPCODE;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true)
        };

        let a = b / c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .div_u32_mut()
            .operations
            .push(Operation::Div32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}
