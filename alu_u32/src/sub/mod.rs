extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{Sub32Cols, NUM_SUB_COLS, SUB_COL_MAP};
use core::mem::transmute;
use valida_bus::{MachineWithGeneralBus, MachineWithRangeBus8};
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};
use valida_opcodes::SUB32;
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
    Sub32(Word<u8>, Word<u8>, Word<u8>),
}

#[derive(Default)]
pub struct Sub32Chip {
    pub operations: Vec<Operation>,
}

impl<F, M> Chip<M> for Sub32Chip
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
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_SUB_COLS);

        pad_to_power_of_two::<NUM_SUB_COLS, F>(&mut trace.values);

        trace
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let sends = SUB_COL_MAP
            .output
            .0
            .map(|field| {
                let output = VirtualPairCol::single_main(field);
                Interaction {
                    fields: vec![output],
                    count: VirtualPairCol::single_main(SUB_COL_MAP.is_real),
                    argument_index: machine.range_bus(),
                }
            })
            .into_iter()
            .collect::<Vec<_>>();
        sends
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::constant(M::F::from_canonical_u32(SUB32));
        let input_1 = SUB_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = SUB_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = SUB_COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(SUB_COL_MAP.is_real),
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl Sub32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_SUB_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::zero(); NUM_SUB_COLS];
        let cols: &mut Sub32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Sub32(a, b, c) => {
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);

                if b[3] < c[3] {
                    cols.borrow[0] = F::one();
                }
                if b[2] < c[2] {
                    cols.borrow[1] = F::one();
                }
                if b[1] < c[1] {
                    cols.borrow[2] = F::one();
                }
                cols.is_real = F::one();
            }
        }
        row
    }
}

pub trait MachineWithSub32Chip: MachineWithCpuChip {
    fn sub_u32(&self) -> &Sub32Chip;
    fn sub_u32_mut(&mut self) -> &mut Sub32Chip;
}

instructions!(Sub32Instruction);

impl<M> Instruction<M> for Sub32Instruction
where
    M: MachineWithSub32Chip + MachineWithRangeChip<256>,
{
    const OPCODE: u32 = SUB32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M>>::OPCODE;
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

        let a = b - c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .sub_u32_mut()
            .operations
            .push(Operation::Sub32(a, b, c));
        state.cpu_mut().push_bus_op(imm, opcode, ops);

        state.range_check(a);
    }
}
