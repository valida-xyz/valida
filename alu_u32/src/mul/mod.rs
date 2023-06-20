extern crate alloc;

use crate::Mul32Opcode;
use alloc::vec;
use alloc::vec::Vec;
use columns::{Mul32Cols, MUL_COL_MAP, NUM_MUL_COLS};
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{
    instructions, Chip, Instruction, Interaction, Operands, PermutationPublicInput, Word,
};
use valida_range::MachineWithRangeChip;

use p3_air::VirtualPairCol;
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

pub struct Mul32PublicInput<F: PrimeField> {
    cumulative_sum: F,
}

impl<F: PrimeField> PermutationPublicInput<F> for Mul32PublicInput<F> {
    fn cumulative_sum(&self) -> F {
        self.cumulative_sum
    }
}

impl<M> Chip<M> for Mul32Chip
where
    M: MachineWithGeneralBus,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_MUL_COLS)
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::single_main(MUL_COL_MAP.opcode);
        let input_1 = MUL_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = MUL_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = MUL_COL_MAP.output.0.map(VirtualPairCol::single_main);

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

    fn local_sends(&self) -> Vec<Interaction<M::F>> {
        todo!()
    }
}

impl Mul32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_MUL_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::ZERO; NUM_MUL_COLS];
        let mut cols: &mut Mul32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Mul32(a, b, c) => {
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
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
    M: MachineWithMul32Chip + MachineWithRangeChip,
{
    const OPCODE: u32 = Mul32Opcode;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c: Word<u8> = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
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
        state.cpu_mut().push_bus_op(imm);

        state.range_record(a);
    }
}
