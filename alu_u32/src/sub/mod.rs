extern crate alloc;

use crate::SUB32_OPCODE;
use alloc::vec;
use alloc::vec::Vec;
use columns::{Sub32Cols, NUM_SUB_COLS, SUB_COL_MAP};
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};
use valida_range::MachineWithRangeChip;

use p3_air::VirtualPairCol;
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
    M: MachineWithGeneralBus,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_SUB_COLS)
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::single_main(SUB_COL_MAP.opcode);
        let input_1 = SUB_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = SUB_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = SUB_COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::one(),
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl Sub32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_SUB_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::ZERO; NUM_SUB_COLS];
        let mut cols: &mut Sub32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Sub32(a, b, c) => {
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
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
    M: MachineWithSub32Chip + MachineWithRangeChip,
{
    const OPCODE: u32 = SUB32_OPCODE;

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

        let a = b - c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .sub_u32_mut()
            .operations
            .push(Operation::Sub32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}
