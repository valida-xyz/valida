extern crate alloc;

use crate::pad_to_power_of_two;
use alloc::vec;
use alloc::vec::Vec;
use columns::{Div32Cols, DIV_COL_MAP, NUM_DIV_COLS};
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};
use valida_opcodes::DIV32;
use valida_range::MachineWithRangeChip;

use p3_air::VirtualPairCol;
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    // (Quotient, Dividend, Divisor)
    // Quotient = Dividend / Divisor
    // It only supports floor division.
    Div32(Word<u8>, Word<u8>, Word<u8>),
}

#[derive(Default)]
pub struct Div32Chip {
    pub clock: u32,
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

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::constant(M::F::from_canonical_u32(DIV32));
        let input_1 = DIV_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = DIV_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = DIV_COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(DIV_COL_MAP.is_real),
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }

    fn local_sends(&self) -> Vec<Interaction<M::F>> {
        // TODO
        vec![]
    }
}

impl Div32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_DIV_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::ZERO; NUM_DIV_COLS];
        let cols: &mut Div32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Div32(a, b, c) => {
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
        }
        row
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
    const OPCODE: u32 = DIV32;

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
