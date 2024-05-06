extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{Lt32Cols, LT_COL_MAP, NUM_LT_COLS};
use core::iter;
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{
    instructions, Chip, Instruction, Interaction, Operands, Word, MEMORY_CELL_BYTES,
};
use valida_opcodes::{LT32, LTE32, SLE32, SLT32};

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::*;

use valida_machine::StarkConfig;
use valida_util::pad_to_power_of_two;
pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Lt32(Word<u8>, Word<u8>, Word<u8>),  // (dst, src1, src2)
    Lte32(Word<u8>, Word<u8>, Word<u8>), // (dst, src1, src2)
    Slt32(Word<u8>, Word<u8>, Word<u8>), // (dst, src1, src2)
    Sle32(Word<u8>, Word<u8>, Word<u8>), // (dst, src1, src2)
}

#[derive(Default)]
pub struct Lt32Chip {
    pub operations: Vec<Operation>,
}

impl<M, SC> Chip<M, SC> for Lt32Chip
where
    M: MachineWithGeneralBus<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();

        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_LT_COLS);

        pad_to_power_of_two::<NUM_LT_COLS, SC::Val>(&mut trace.values);

        trace
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let opcode = VirtualPairCol::new_main(
            vec![
                (LT_COL_MAP.is_lt, SC::Val::from_canonical_u32(LT32)),
                (LT_COL_MAP.is_lte, SC::Val::from_canonical_u32(LTE32)),
                (LT_COL_MAP.is_slt, SC::Val::from_canonical_u32(SLT32)),
                (LT_COL_MAP.is_sle, SC::Val::from_canonical_u32(SLE32)),
            ],
            SC::Val::zero(),
        );
        let input_1 = LT_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = LT_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = (0..MEMORY_CELL_BYTES - 1)
            .map(|_| VirtualPairCol::constant(SC::Val::zero()))
            .chain(iter::once(VirtualPairCol::single_main(LT_COL_MAP.output)));

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(LT_COL_MAP.multiplicity),
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl Lt32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_LT_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::zero(); NUM_LT_COLS];
        let cols: &mut Lt32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Lt32(a, b, c) => {
                cols.is_lt = F::one();
                self.set_cols(cols, false, a, b, c);
            }
            Operation::Lte32(a, b, c) => {
                cols.is_lte = F::one();
                self.set_cols(cols, false, a, b, c);
            }
            Operation::Slt32(a, b, c) => {
                cols.is_slt = F::one();
                self.set_cols(cols, true, a, b, c);
            }
            Operation::Sle32(a, b, c) => {
                cols.is_sle = F::one();
                self.set_cols(cols, true, a, b, c);
            }
        }
        row
    }

    fn set_cols<F>(
        &self,
        cols: &mut Lt32Cols<F>,
        is_signed: bool,
        a: &Word<u8>,
        b: &Word<u8>,
        c: &Word<u8>,
    ) where
        F: PrimeField,
    {
        // Set the input columns
        debug_assert_eq!(a.0.len(), 4);
        debug_assert_eq!(b.0.len(), 4);
        debug_assert_eq!(c.0.len(), 4);
        cols.input_1 = b.transform(F::from_canonical_u8);
        cols.input_2 = c.transform(F::from_canonical_u8);
        cols.output = F::from_canonical_u8(a[3]);

        if let Some(n) = b
            .into_iter()
            .zip(c.into_iter())
            .enumerate()
            .find_map(|(n, (x, y))| if x == y { None } else { Some(n) })
        {
            let z = 256u16 + b[n] as u16 - c[n] as u16;
            for i in 0..9 {
                cols.bits[i] = F::from_canonical_u16(z >> i & 1);
            }
            cols.byte_flag[n] = F::one();
            // b[n] != c[n] always here, so the difference is never zero.
            cols.diff_inv = (cols.input_1[n] - cols.input_2[n]).inverse();
        }
        // compute (little-endian) bit decomposition of the top bytes
        for i in 0..8 {
            cols.top_bits_1[i] = F::from_canonical_u8(b[0] >> i & 1);
            cols.top_bits_2[i] = F::from_canonical_u8(c[0] >> i & 1);
        }
        // check if sign bits agree and set different_signs accordingly
        cols.different_signs = if is_signed {
            if cols.top_bits_1[7] != cols.top_bits_2[7] {
                F::one()
            } else {
                F::zero()
            }
        } else {
            F::zero()
        };

        cols.multiplicity = F::one();
    }

    fn execute_with_closure<M, E, F>(
        state: &mut M,
        ops: Operands<i32>,
        opcode: u32,
        comp: F,
    ) -> (Word<u8>, Word<u8>, Word<u8>)
    where
        M: MachineWithLt32Chip<E>,
        E: Field,
        F: Fn(Word<u8>, Word<u8>) -> bool,
    {
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let src1: Word<u8> = if ops.d() == 1 {
            let b = (ops.b() as u32).into();
            imm = Some(b);
            b
        } else {
            let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_1, true, pc, opcode, 0, "")
        };
        let src2: Word<u8> = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        let dst = if comp(src1, src2) {
            Word::from(1)
        } else {
            Word::from(0)
        };
        state.mem_mut().write(clk, write_addr, dst, true);

        if ops.d() == 1 {
            state.cpu_mut().push_left_imm_bus_op(imm, opcode, ops)
        } else {
            state.cpu_mut().push_bus_op(imm, opcode, ops);
        }
        (dst, src1, src2)
    }
}

pub trait MachineWithLt32Chip<F: Field>: MachineWithCpuChip<F> {
    fn lt_u32(&self) -> &Lt32Chip;
    fn lt_u32_mut(&mut self) -> &mut Lt32Chip;
}

instructions!(
    Lt32Instruction,
    Lte32Instruction,
    Slt32Instruction,
    Sle32Instruction
);

impl<M, F> Instruction<M, F> for Lt32Instruction
where
    M: MachineWithLt32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = LT32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let comp = |a, b| a < b;
        let (dst, src1, src2) = Lt32Chip::execute_with_closure(state, ops, opcode, comp);
        state
            .lt_u32_mut()
            .operations
            .push(Operation::Lt32(dst, src1, src2));
    }
}

impl<M, F> Instruction<M, F> for Lte32Instruction
where
    M: MachineWithLt32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = LTE32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let comp = |a, b| a <= b;
        let (dst, src1, src2) = Lt32Chip::execute_with_closure(state, ops, opcode, comp);
        state
            .lt_u32_mut()
            .operations
            .push(Operation::Lte32(dst, src1, src2));
    }
}

impl<M, F> Instruction<M, F> for Slt32Instruction
where
    M: MachineWithLt32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = SLT32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let comp = |a: Word<u8>, b: Word<u8>| {
            let a_i: i32 = a.into();
            let b_i: i32 = b.into();
            a_i < b_i
        };
        let (dst, src1, src2) = Lt32Chip::execute_with_closure(state, ops, opcode, comp);
        state
            .lt_u32_mut()
            .operations
            .push(Operation::Slt32(dst, src1, src2));
    }
}

impl<M, F> Instruction<M, F> for Sle32Instruction
where
    M: MachineWithLt32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = SLE32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let comp = |a: Word<u8>, b: Word<u8>| {
            let a_i: i32 = a.into();
            let b_i: i32 = b.into();
            a_i <= b_i
        };
        let (dst, src1, src2) = Lt32Chip::execute_with_closure(state, ops, opcode, comp);
        state
            .lt_u32_mut()
            .operations
            .push(Operation::Sle32(dst, src1, src2));
    }
}
