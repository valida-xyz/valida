extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{Add32Cols, ADD_COL_MAP, NUM_ADD_COLS};
use core::mem::transmute;
use valida_bus::{MachineWithGeneralBus, MachineWithRangeBus8};
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};
use valida_opcodes::ADD32;
use valida_range::MachineWithRangeChip;

use p3_air::VirtualPairCol;
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;
use valida_util::pad_to_power_of_two;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Add32(Word<u8>, Word<u8>, Word<u8>),
}

#[derive(Default)]
pub struct Add32Chip {
    pub operations: Vec<Operation>,
}

impl<F, M> Chip<M> for Add32Chip
where
    F: PrimeField,
    M: MachineWithGeneralBus<F = F> + MachineWithRangeBus8,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();

        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_ADD_COLS);

        pad_to_power_of_two::<NUM_ADD_COLS, F>(&mut trace.values);

        trace
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let sends = ADD_COL_MAP
            .output
            .0
            .map(|field| {
                let output = VirtualPairCol::single_main(field);
                Interaction {
                    fields: vec![output],
                    count: VirtualPairCol::single_main(ADD_COL_MAP.is_real),
                    argument_index: machine.range_bus(),
                }
            })
            .into_iter()
            .collect::<Vec<_>>();
        sends
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::constant(M::F::from_canonical_u32(ADD32));
        let input_1 = ADD_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = ADD_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = ADD_COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(ADD_COL_MAP.is_real),
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl Add32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_ADD_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::zero(); NUM_ADD_COLS];
        let cols: &mut Add32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Add32(a, b, c) => {
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);

                let mut carry_1 = 0;
                let mut carry_2 = 0;
                if b[3] as u32 + c[3] as u32 > 255 {
                    carry_1 = 1;
                    cols.carry[0] = F::one();
                }
                if b[2] as u32 + c[2] as u32 + carry_1 > 255 {
                    carry_2 = 1;
                    cols.carry[1] = F::one();
                }
                if b[1] as u32 + c[1] as u32 + carry_2 > 255 {
                    cols.carry[2] = F::one();
                }
                cols.is_real = F::one();
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
    M: MachineWithAdd32Chip + MachineWithRangeChip<256>,
{
    const OPCODE: u32 = ADD32;

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
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}
