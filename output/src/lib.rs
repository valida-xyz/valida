use crate::columns::{OutputCols, NUM_OUTPUT_COLS};
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Operands, Word};

use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

const WRITE_OPCODE: u32 = 102;

pub mod columns;
pub mod stark;

pub struct OutputChip {
    buffer: Vec<Word<u8>>,
}

impl<F, M> Chip<M> for OutputChip
where
    F: PrimeField,
    M: MachineWithGeneralBus<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .buffer
            .par_iter()
            .enumerate()
            .map(|(n, value)| {
                let mut row = [M::F::ZERO; NUM_OUTPUT_COLS];
                let mut cols: &mut OutputCols<M::F> = unsafe { transmute(&mut row) };

                cols.addr = M::F::from_canonical_u32(n as u32);
                cols.value = value.transform(M::F::from_canonical_u8);
                row
            })
            .flatten()
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows, NUM_OUTPUT_COLS)
    }
}

pub trait MachineWithOutputChip: MachineWithCpuChip {
    fn output(&self) -> &OutputChip;
    fn output_mut(&mut self) -> &mut OutputChip;
}

instructions!(WriteInstruction);

impl<M> Instruction<M> for WriteInstruction
where
    M: MachineWithOutputChip,
{
    const OPCODE: u32 = WRITE_OPCODE;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        state.output_mut().buffer.push(b);

        // The assigned output address is sent back to the CPU
        let a = state.output().buffer.len() - 1;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        state
            .mem_mut()
            .write(clk, write_addr, Word::from(a as u32), true);

        // The immediate value flag should be set, and the immediate operand value should
        // equal zero. We only write one word at a time to the output buffer.
        assert_eq!(ops.is_imm(), 1);
        assert_eq!(ops.c(), 0);
    }
}
