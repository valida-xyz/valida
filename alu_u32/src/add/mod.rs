extern crate alloc;

use super::Add32Opcode;
use alloc::vec;
use alloc::vec::Vec;
use columns::{Add32Cols, ADD_COL_MAP, NUM_ADD_COLS};
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};

use p3_air::VirtualPairCol;
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Add32(Word<u8>, Word<u8>, Word<u8>),
}

#[derive(Default)]
pub struct Add32Chip {
    pub clock: u32,
    pub operations: Vec<Operation>,
}

impl<M> Chip<M> for Add32Chip
where
    M: MachineWithAdd32Chip + MachineWithGeneralBus,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row::<M::F, M>(op))
            .flatten()
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows, NUM_ADD_COLS)
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::single_main(ADD_COL_MAP.opcode);
        let input_1 = ADD_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = ADD_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = ADD_COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(ADD_COL_MAP.opcode),
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl Add32Chip {
    fn op_to_row<F, M>(&self, op: &Operation) -> [F; NUM_ADD_COLS]
    where
        F: PrimeField,
        M: MachineWithAdd32Chip<F = F>,
    {
        let mut row = [F::ZERO; NUM_ADD_COLS];
        let mut cols: &mut Add32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Add32(a, b, c) => {
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);
            }
        }
        row
    }
}

pub trait MachineWithAdd32Chip: MachineWithCpuChip {
    fn add_u32(&self) -> &Add32Chip;
    fn add_u32_mut(&mut self) -> &mut Add32Chip;
}

instructions!(Add32Instruction);

impl<M> Instruction<M> for Add32Instruction
where
    M: MachineWithAdd32Chip,
{
    const OPCODE: u32 = Add32Opcode;

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

        let a = b + c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .add_u32_mut()
            .operations
            .push(Operation::Add32(a, b, c));
        state.cpu_mut().push_bus_op(imm);
    }
}
