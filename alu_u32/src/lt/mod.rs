extern crate alloc;

use crate::pad_to_power_of_two;
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
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

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

impl<F, M> Chip<M> for Lt32Chip
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
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_LT_COLS);

        pad_to_power_of_two::<NUM_LT_COLS, F>(&mut trace.values);

        trace
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::constant(M::F::from_canonical_u32(LT32));
        let input_1 = LT_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = LT_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = (0..MEMORY_CELL_BYTES - 1)
            .map(|_| VirtualPairCol::constant(M::F::ZERO))
            .chain(iter::once(VirtualPairCol::single_main(LT_COL_MAP.output)))
            .collect::<Vec<_>>();

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(LT_COL_MAP.is_real),
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
        let mut row = [F::ZERO; NUM_LT_COLS];
        let cols: &mut Lt32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Lt32(a, b, c) => {
                if let Some(n) = b
                    .into_iter()
                    .zip(c.into_iter())
                    .enumerate()
                    .find_map(|(n, (x, y))| if x == y { Some(n) } else { None })
                {
                    let z = 128 + b[n] - c[n];
                    for i in 0..9 {
                        cols.bits[i] = F::from_canonical_u8(z >> i & 1);
                    }
                    if n < 3 {
                        cols.byte_flag[n] = F::ONE;
                    }
                }
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = F::from_canonical_u8(a[3]);
                cols.is_real = F::ONE;
            }
        }
        row
    }
}

pub trait MachineWithLt32Chip: MachineWithCpuChip {
    fn add_u32(&self) -> &Lt32Chip;
    fn add_u32_mut(&mut self) -> &mut Lt32Chip;
}

instructions!(Lt32Instruction);

impl<M> Instruction<M> for Lt32Instruction
where
    M: MachineWithLt32Chip,
{
    const OPCODE: u32 = LT32;

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

        let a = if b < c { Word::from(1) } else { Word::from(0) };
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .add_u32_mut()
            .operations
            .push(Operation::Lt32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);
    }
}
