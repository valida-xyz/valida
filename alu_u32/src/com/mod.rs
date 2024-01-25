extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{Com32Cols, COM_COL_MAP, NUM_COM_COLS};
use core::iter;
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::StarkConfig;
use valida_machine::{
    instructions, Chip, Instruction, Interaction, Operands, Word, MEMORY_CELL_BYTES,
};
use valida_opcodes::{EQ32, NE32};

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
// use p3_maybe_rayon::*;
use p3_maybe_rayon::prelude::IntoParallelRefIterator;
use valida_util::pad_to_power_of_two;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Ne32(Word<u8>, Word<u8>, Word<u8>), // (dst, src1, src2)
    Eq32(Word<u8>, Word<u8>, Word<u8>), // (dst, src1, src2)
}

#[derive(Default)]
pub struct Com32Chip {
    pub operations: Vec<Operation>,
}

impl<M, SC> Chip<M, SC> for Com32Chip
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
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COM_COLS);

        pad_to_power_of_two::<NUM_COM_COLS, SC::Val>(&mut trace.values);

        trace
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let opcode = VirtualPairCol::new_main(
            vec![
                (COM_COL_MAP.is_ne, SC::Val::from_canonical_u32(NE32)),
                (COM_COL_MAP.is_eq, SC::Val::from_canonical_u32(EQ32)),
            ],
            SC::Val::zero(),
        );
        let input_1 = COM_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = COM_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = (0..MEMORY_CELL_BYTES - 1)
            .map(|_| VirtualPairCol::constant(SC::Val::zero()))
            .chain(iter::once(VirtualPairCol::single_main(COM_COL_MAP.output)));

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let is_real = VirtualPairCol::sum_main(vec![COM_COL_MAP.is_ne, COM_COL_MAP.is_eq]);

        let receive = Interaction {
            fields,
            count: is_real,
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl Com32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_COM_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::zero(); NUM_COM_COLS];
        let cols: &mut Com32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Ne32(_, _, _) => {
                cols.is_ne = F::one();
            }
            Operation::Eq32(_, _, _) => {
                cols.is_eq = F::one();
            }
        }
        row
    }
}

pub trait MachineWithCom32Chip<F: Field>: MachineWithCpuChip<F> {
    fn com_u32(&self) -> &Com32Chip;
    fn com_u32_mut(&mut self) -> &mut Com32Chip;
}

instructions!(Ne32Instruction, Eq32Instruction);

impl<M, F> Instruction<M, F> for Ne32Instruction
where
    M: MachineWithCom32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = NE32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let src1 = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
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

        let dst = if src1 != src2 {
            Word::from(1)
        } else {
            Word::from(0)
        };
        state.mem_mut().write(clk, write_addr, dst, true);

        state
            .com_u32_mut()
            .operations
            .push(Operation::Ne32(dst, src1, src2));
        state.cpu_mut().push_bus_op(imm, opcode, ops);
    }
}

impl<M, F> Instruction<M, F> for Eq32Instruction
where
    M: MachineWithCom32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = EQ32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let src1 = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
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

        let dst = if src1 == src2 {
            Word::from(1)
        } else {
            Word::from(0)
        };
        state.mem_mut().write(clk, write_addr, dst, true);

        state
            .com_u32_mut()
            .operations
            .push(Operation::Eq32(dst, src1, src2));
        state.cpu_mut().push_bus_op(imm, opcode, ops);
    }
}
