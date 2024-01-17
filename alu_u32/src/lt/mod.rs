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
use valida_opcodes::LT32;

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::*;
use valida_machine::config::StarkConfig;
use valida_util::pad_to_power_of_two;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Lt32(Word<u8>, Word<u8>, Word<u8>), // (dst, src1, src2)
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
        let opcode = VirtualPairCol::constant(SC::Val::from_canonical_u32(LT32));
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
            Operation::Lt32(dst, src1, src2) => {
                if let Some(n) = src1
                    .into_iter()
                    .zip(src2.into_iter())
                    .enumerate()
                    .find_map(|(n, (x, y))| if x == y { Some(n) } else { None })
                {
                    let z = 256u16 + src1[n] as u16 - src2[n] as u16;
                    for i in 0..10 {
                        cols.bits[i] = F::from_canonical_u16(z >> i & 1);
                    }
                    if n < 3 {
                        cols.byte_flag[n] = F::one();
                    }
                }
                cols.input_1 = src1.transform(F::from_canonical_u8);
                cols.input_2 = src2.transform(F::from_canonical_u8);
                cols.output = F::from_canonical_u8(dst[3]);
                cols.multiplicity = F::one();
            }
        }
        row
    }
}

pub trait MachineWithLt32Chip<F: Field>: MachineWithCpuChip<F> {
    fn lt_u32(&self) -> &Lt32Chip;
    fn lt_u32_mut(&mut self) -> &mut Lt32Chip;
}

instructions!(Lt32Instruction);

impl<M, F> Instruction<M, F> for Lt32Instruction
where
    M: MachineWithLt32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = LT32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let src1 = if ops.d() == 1 {
            let b = (ops.b() as u32).into();
            imm = Some(b);
            b
        } else {
            state
                .mem_mut()
                .read(clk, read_addr_1, true, pc, opcode, 0, "")
        };
        let src2 = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        let dst = if src1 < src2 {
            Word::from(1)
        } else {
            Word::from(0)
        };
        state.mem_mut().write(clk, write_addr, dst, true);

        state
            .lt_u32_mut()
            .operations
            .push(Operation::Lt32(dst, src1, src2));
        state.cpu_mut().push_bus_op(imm, opcode, ops);
    }
}
