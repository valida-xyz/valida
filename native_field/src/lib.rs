#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{NativeFieldCols, COL_MAP, NUM_COLS};
use core::mem::transmute;
use valida_bus::{MachineWithGeneralBus, MachineWithRangeBus8};
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Machine, Operands, Word};
use valida_range::MachineWithRangeChip;

use p3_air::VirtualPairCol;
use p3_field::{Field, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

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

pub struct NativeFieldChip {
    operations: Vec<Operation>,
}

impl<F, M> Chip<M> for NativeFieldChip
where
    F: Field,
    M: MachineWithGeneralBus<F = F> + MachineWithRangeBus8,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();

        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS);

        Self::pad_to_power_of_two::<NUM_COLS, M::F>(&mut trace.values);

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
        let opcode = VirtualPairCol::new_main(
            vec![
                (COL_MAP.is_add, M::F::from_canonical_u32(ADD_OPCODE)),
                (COL_MAP.is_sub, M::F::from_canonical_u32(SUB_OPCODE)),
                (COL_MAP.is_mul, M::F::from_canonical_u32(MUL_OPCODE)),
            ],
            M::F::ZERO,
        );
        let input_1 = COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let is_real = VirtualPairCol::new_main(
            vec![
                (COL_MAP.is_add, M::F::ONE),
                (COL_MAP.is_sub, M::F::ONE),
                (COL_MAP.is_mul, M::F::ONE),
            ],
            M::F::ZERO,
        );

        let receive = Interaction {
            fields,
            count: is_real,
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl NativeFieldChip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_COLS]
    where
        F: Field,
    {
        let mut row = [F::ZERO; NUM_COLS];
        let cols: &mut NativeFieldCols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Add(a, b, c) => {
                cols.is_add = F::ONE;
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
            Operation::Sub(a, b, c) => {
                cols.is_sub = F::ONE;
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
            Operation::Mul(a, b, c) => {
                cols.is_mul = F::ONE;
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
        }

        row
    }

    fn pad_to_power_of_two<const N: usize, F: Field>(values: &mut Vec<F>) {
        let n_real_rows = values.len() / N;
        values.resize(n_real_rows.next_power_of_two() * N, F::ZERO);
    }
}

pub trait MachineWithNativeFieldChip: MachineWithCpuChip {
    fn native_field(&self) -> NativeFieldChip;
    fn native_field_mut(&self) -> &mut NativeFieldChip;
}

instructions!(AddInstruction, SubInstruction, MulInstruction);

impl<F, M> Instruction<M> for AddInstruction
where
    M: MachineWithNativeFieldChip + MachineWithRangeChip + Machine<F = F>,
    F: PrimeField32,
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

        let a_native = F::from_canonical_u32(b.into()) + F::from_canonical_u32(c.into());
        let a = Word::from(a_native.as_canonical_u32());
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .native_field_mut()
            .operations
            .push(Operation::Add(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}

impl<F, M> Instruction<M> for SubInstruction
where
    M: MachineWithNativeFieldChip + MachineWithRangeChip + Machine<F = F>,
    F: PrimeField32,
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

        let a_native = F::from_canonical_u32(b.into()) - F::from_canonical_u32(c.into());
        let a = Word::from(a_native.as_canonical_u32());
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .native_field_mut()
            .operations
            .push(Operation::Sub(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}

impl<F, M> Instruction<M> for MulInstruction
where
    M: MachineWithNativeFieldChip + MachineWithRangeChip + Machine<F = F>,
    F: PrimeField32,
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

        let a_m31 = M::F::from_canonical_u32(b.into()) * M::F::from_canonical_u32(c.into());
        let a = Word::from(a_m31.as_canonical_u32());
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .native_field()
            .operations
            .push(Operation::Mul(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}
