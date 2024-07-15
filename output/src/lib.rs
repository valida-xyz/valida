use crate::columns::{OutputCols, NUM_OUTPUT_COLS, OUTPUT_COL_MAP};
use core::mem::transmute;
use valida_bus::{MachineWithGeneralBus, MachineWithOutputBus, MachineWithRangeBus8};
use valida_cpu::{MachineWithCpuChip, Operation};
use valida_machine::{
    instructions, Chip, Instruction, Interaction, Operands, Word, MEMORY_CELL_BYTES,
};
use valida_opcodes::WRITE;

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::*;
use valida_machine::StarkConfig;
use valida_range::MachineWithRangeChip;
use valida_util::pad_to_power_of_two;

pub mod columns;
pub mod stark;

#[derive(Default)]
pub struct OutputChip {
    pub values: Vec<(u32, u8)>, // (clk, byte)
}

impl OutputChip {
    pub fn bytes(&self) -> Vec<u8> {
        self.values.iter().map(|(_, b)| *b).collect()
    }
}

impl<M, SC> Chip<M, SC> for OutputChip
where
    M: MachineWithGeneralBus<SC::Val>
        + MachineWithRangeBus8<SC::Val>
        + MachineWithOutputBus<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        let mut rows = self
            .values
            .as_slice()
            .par_windows(2)
            .map(|window| {
                let clk_1 = window[0].0;
                let clk_2 = window[1].0;
                let val_1 = window[0].1;
                let clk_diff: Word<u8> = (clk_2 - clk_1).into();
                let mut row = [SC::Val::zero(); NUM_OUTPUT_COLS];
                let cols: &mut OutputCols<SC::Val> = unsafe { transmute(&mut row) };
                cols.clk = SC::Val::from_canonical_u32(clk_1);
                cols.value = SC::Val::from_canonical_u8(val_1);
                cols.diff = clk_diff.transform(SC::Val::from_canonical_u8);
                cols.is_real = SC::Val::one();
                row
            })
            .collect::<Vec<_>>();

        // Add final row
        if let Some(last_row) = self.values.last() {
            let mut row = [SC::Val::zero(); NUM_OUTPUT_COLS];
            let cols: &mut OutputCols<SC::Val> = unsafe { transmute(&mut row) };
            cols.clk = SC::Val::from_canonical_u32(last_row.0);
            cols.value = SC::Val::from_canonical_u8(last_row.1);
            cols.is_real = SC::Val::one();
            rows.push(row);
        }

        let mut trace = RowMajorMatrix::new(
            rows.into_iter().flatten().collect::<Vec<_>>(),
            NUM_OUTPUT_COLS,
        );

        pad_to_power_of_two::<NUM_OUTPUT_COLS, SC::Val>(&mut trace.values);

        trace
    }

    /// To ensure that the output is correctly sorted, we check that the difference between consecutive
    /// clock values is in the range 0..2^31.
    fn global_sends(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let mut diff_bytes: Vec<_> = (0..MEMORY_CELL_BYTES)
            .map(|i| VirtualPairCol::single_main(OUTPUT_COL_MAP.diff[i]))
            .collect();
        let twice_top_byte = VirtualPairCol::new_main(
            vec![(OUTPUT_COL_MAP.diff[0], SC::Val::two())],
            SC::Val::zero(),
        );
        diff_bytes.push(twice_top_byte);
        diff_bytes
            .into_iter()
            .map(|byte| Interaction {
                fields: vec![byte],
                count: VirtualPairCol::single_main(OUTPUT_COL_MAP.is_real),
                argument_index: machine.range_bus(),
            })
            .collect()
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let clk = VirtualPairCol::single_main(OUTPUT_COL_MAP.clk);
        let value = VirtualPairCol::single_main(OUTPUT_COL_MAP.value);

        let fields = vec![clk, value];

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(OUTPUT_COL_MAP.is_real),
            argument_index: machine.output_bus(),
        };
        vec![receive]
    }
}

pub trait MachineWithOutputChip<F: Field>: MachineWithCpuChip<F> {
    fn output(&self) -> &OutputChip;
    fn output_mut(&mut self) -> &mut OutputChip;
}

instructions!(WriteInstruction);

impl<M, F> Instruction<M, F> for WriteInstruction
where
    M: MachineWithOutputChip<F> + MachineWithRangeChip<F, 256>,
    F: Field,
{
    const OPCODE: u32 = WRITE;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let b = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let last_clk = state.output().values.last().map(|(c, _)| *c).unwrap_or(clk);
        state
            .output_mut()
            .values
            .push((clk, b.into_iter().last().unwrap()));
        // The range check counter should be updated.
        let clk_diff: Word<u8> = (clk - last_clk).into();
        state.range_check(clk_diff);
        let twice_top_byte_of_diff: u8 = (clk_diff.0[0] as u16 * 2)
            .try_into()
            .expect("top bit of diff should be 0");
        state
            .range_mut()
            .count
            .entry(twice_top_byte_of_diff as u32)
            .and_modify(|c| *c += 1)
            .or_insert(1);

        state.cpu_mut().pc += 1;
        state.cpu_mut().push_op(Operation::Write, opcode, ops);

        // The immediate value flag should be set, and the immediate operand value should
        // equal zero. We only write one byte of one word at a time to output.
        assert_eq!(ops.is_imm(), 1);
        assert_eq!(ops.c(), 0);
    }
}
