extern crate alloc;

use crate::pad_to_power_of_two;
use alloc::vec;
use alloc::vec::Vec;
use columns::{Mersenne31Cols, COL_MAP, NUM_COLS};
use core::mem::transmute;
use valida_bus::{MachineWithGeneralBus, MachineWithRangeBus8};
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};
use valida_range::MachineWithRangeChip;

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;
use p3_mersenne_31::Mersenne31;

const ADD_OPCODE: u32 = 13;
const SUB_OPCODE: u32 = 14;
const MUL_OPCODE: u32 = 15;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Add(Word<u8>, Word<u8>, Word<u8>),
    Sub(Word<u8>, Word<u8>, Word<u8>),
    Mul(Word<u8>, Word<u8>, Word<u8>),
}

pub struct Mersenne31Chip {
    operations: Vec<Operation>,
}

impl<M> Chip<M> for Mersenne31Chip
where
    M: MachineWithGeneralBus + MachineWithRangeBus8,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();

        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS);

        pad_to_power_of_two::<NUM_COLS, M::F>(&mut trace.values);

        trace
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let output = COL_MAP
            .output
            .0
            .map(VirtualPairCol::single_main)
            .into_iter()
            .collect::<Vec<_>>();

        let send = Interaction {
            fields: output,
            count: VirtualPairCol::one(),
            argument_index: machine.range_bus(),
        };
        vec![send]
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::single_main(COL_MAP.opcode);
        let input_1 = COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(COL_MAP.is_real),
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl Mersenne31Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::ZERO; NUM_COLS];
        let cols: &mut Mersenne31Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Add(a, b, c) => {
                cols.is_add = F::ONE;
                cols.opcode = F::from_canonical_u32(ADD_OPCODE);
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
            Operation::Sub(a, b, c) => {
                cols.is_sub = F::ONE;
                cols.opcode = F::from_canonical_u32(SUB_OPCODE);
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
            Operation::Mul(a, b, c) => {
                cols.is_mul = F::ONE;
                cols.opcode = F::from_canonical_u32(MUL_OPCODE);
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
        }

        cols.is_real = F::ONE;

        row
    }
}

pub trait MachineWithMersenne31Field: MachineWithCpuChip {
    fn mersenne_31(&self) -> Mersenne31Chip;
    fn mersenne_31_mut(&self) -> &mut Mersenne31Chip;
}

instructions!(AddInstruction, SubInstruction, MulInstruction);

impl<M> Instruction<M> for AddInstruction
where
    M: MachineWithMersenne31Field + MachineWithRangeChip,
{
    const OPCODE: u32 = ADD_OPCODE;

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

        let a_m31 =
            Mersenne31::from_canonical_u32(b.into()) + Mersenne31::from_canonical_u32(c.into());
        let a = Word::from(a_m31.as_noncanonical_u32());
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .mersenne_31_mut()
            .operations
            .push(Operation::Add(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}

impl<M> Instruction<M> for SubInstruction
where
    M: MachineWithMersenne31Field + MachineWithRangeChip,
{
    const OPCODE: u32 = SUB_OPCODE;

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

        let a_m31 =
            Mersenne31::from_canonical_u32(b.into()) - Mersenne31::from_canonical_u32(c.into());
        let a = Word::from(a_m31.as_noncanonical_u32());
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .mersenne_31_mut()
            .operations
            .push(Operation::Sub(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}

impl<M> Instruction<M> for MulInstruction
where
    M: MachineWithMersenne31Field + MachineWithRangeChip,
{
    const OPCODE: u32 = MUL_OPCODE;

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

        let a_m31 =
            Mersenne31::from_canonical_u32(b.into()) * Mersenne31::from_canonical_u32(c.into());
        let a = Word::from(a_m31.as_noncanonical_u32());
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .mersenne_31_mut()
            .operations
            .push(Operation::Mul(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}
