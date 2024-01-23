#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{NativeFieldCols, COL_MAP, NUM_NATIVE_FIELD_COLS};
use core::mem::transmute;
use valida_bus::{MachineWithGeneralBus, MachineWithRangeBus8};
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};
use valida_opcodes::{ADD, MUL, SUB};
use valida_range::MachineWithRangeChip;
use valida_util::pad_to_power_of_two;

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;
use p3_uni_stark::StarkConfig;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Add(Word<u8>, Word<u8>, Word<u8>), // dst, src1, src2
    Sub(Word<u8>, Word<u8>, Word<u8>), // dst, src1, src2
    Mul(Word<u8>, Word<u8>, Word<u8>), // dst, src1, src2
}

pub struct NativeFieldChip {
    operations: Vec<Operation>,
}

impl<M, SC> Chip<M, SC> for NativeFieldChip
where
    M: MachineWithGeneralBus<SC::Val> + MachineWithRangeBus8<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();

        let mut trace = RowMajorMatrix::new(
            rows.into_iter().flatten().collect::<Vec<_>>(),
            NUM_NATIVE_FIELD_COLS,
        );

        pad_to_power_of_two::<NUM_NATIVE_FIELD_COLS, SC::Val>(&mut trace.values);

        trace
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let sends = COL_MAP
            .output
            .0
            .map(|field| {
                let output = VirtualPairCol::single_main(field);
                let is_real =
                    VirtualPairCol::sum_main(vec![COL_MAP.is_add, COL_MAP.is_sub, COL_MAP.is_mul]);

                Interaction {
                    fields: vec![output],
                    count: is_real,
                    argument_index: machine.range_bus(),
                }
            })
            .into_iter()
            .collect::<Vec<_>>();
        sends
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let opcode = VirtualPairCol::new_main(
            vec![
                (COL_MAP.is_add, SC::Val::from_canonical_u32(ADD)),
                (COL_MAP.is_sub, SC::Val::from_canonical_u32(SUB)),
                (COL_MAP.is_mul, SC::Val::from_canonical_u32(MUL)),
            ],
            SC::Val::zero(),
        );
        let input_1 = COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let is_real =
            VirtualPairCol::sum_main(vec![COL_MAP.is_add, COL_MAP.is_sub, COL_MAP.is_mul]);

        let receive = Interaction {
            fields,
            count: is_real,
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl NativeFieldChip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_NATIVE_FIELD_COLS]
    where
        F: Field,
    {
        let mut row = [F::zero(); NUM_NATIVE_FIELD_COLS];
        let cols: &mut NativeFieldCols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Add(a, b, c) => {
                cols.is_add = F::one();
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
            Operation::Sub(a, b, c) => {
                cols.is_sub = F::one();
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
            Operation::Mul(a, b, c) => {
                cols.is_mul = F::one();
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
        }

        row
    }
}

pub trait MachineWithNativeFieldChip<F: Field>: MachineWithCpuChip<F> {
    fn native_field(&self) -> NativeFieldChip;
    fn native_field_mut(&self) -> &mut NativeFieldChip;
}

instructions!(AddInstruction, SubInstruction, MulInstruction);

impl<M, F> Instruction<M, F> for AddInstruction
where
    M: MachineWithNativeFieldChip<F> + MachineWithRangeChip<F, 256>,
    F: PrimeField32,
{
    const OPCODE: u32 = ADD;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        let a_native = F::from_canonical_u32(b.into()) + F::from_canonical_u32(c.into());
        let a = Word::from(a_native.as_canonical_u32());
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .native_field_mut()
            .operations
            .push(Operation::Add(a, b, c));
        state.cpu_mut().push_bus_op(imm, opcode, ops);

        state.range_check(a);
    }
}

impl<M, F> Instruction<M, F> for SubInstruction
where
    M: MachineWithNativeFieldChip<F> + MachineWithRangeChip<F, 256>,
    F: PrimeField32,
{
    const OPCODE: u32 = SUB;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        let a_native = F::from_canonical_u32(b.into()) - F::from_canonical_u32(c.into());
        let a = Word::from(a_native.as_canonical_u32());
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .native_field_mut()
            .operations
            .push(Operation::Sub(a, b, c));
        state.cpu_mut().push_bus_op(imm, opcode, ops);

        state.range_check(a);
    }
}

impl<M, F> Instruction<M, F> for MulInstruction
where
    M: MachineWithNativeFieldChip<F> + MachineWithRangeChip<F, 256>,
    F: PrimeField32,
{
    const OPCODE: u32 = MUL;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        let a_m31 = F::from_canonical_u32(b.into()) * F::from_canonical_u32(c.into());
        let a = Word::from(a_m31.as_canonical_u32());
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .native_field()
            .operations
            .push(Operation::Mul(a, b, c));
        state.cpu_mut().push_bus_op(imm, opcode, ops);

        state.range_check(a);
    }
}
