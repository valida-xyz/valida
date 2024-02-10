use crate::columns::{OutputCols, NUM_OUTPUT_COLS, OUTPUT_COL_MAP};
use core::fmt::Debug;
use core::iter;
use core::mem::transmute;
use proptest::prelude::Arbitrary;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{
    instructions, Chip, Instruction, Interaction, Operands, CPU_MEMORY_CHANNELS, MEMORY_CELL_BYTES,
};
use valida_opcodes::WRITE;

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::*;
use valida_machine::StarkConfig;
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
    M: MachineWithGeneralBus<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        let table_len = self.values.len() as u32;
        let mut rows = self
            .values
            .as_slice()
            .par_windows(2)
            .map(|window| {
                let clk_1 = window[0].0;
                let clk_2 = window[1].0;
                let val_1 = window[0].1;
                let clk_diff = clk_2 - clk_1;
                let num_rows = (clk_diff / table_len) as usize + 1;
                let mut rows = Vec::with_capacity(num_rows);
                for i in 0..num_rows {
                    let mut row = [SC::Val::zero(); NUM_OUTPUT_COLS];
                    let cols: &mut OutputCols<SC::Val> = unsafe { transmute(&mut row) };
                    if i == 0 {
                        cols.is_real = SC::Val::one();
                        cols.clk = SC::Val::from_canonical_u32(clk_1);
                        cols.value = SC::Val::from_canonical_u8(val_1);
                    } else {
                        // Dummy output to satisfy range check
                        cols.clk = SC::Val::from_canonical_u32(clk_1 + table_len * (i + 1) as u32);
                    }
                    rows.push(row);
                }

                // Compute clock diffs
                rows.iter()
                    .map(|row| row[OUTPUT_COL_MAP.clk])
                    .chain(iter::once(SC::Val::from_canonical_u32(clk_2)))
                    .collect::<Vec<_>>()
                    .windows(2)
                    .enumerate()
                    .for_each(|(n, clks)| {
                        let cols: &mut OutputCols<SC::Val> = unsafe { transmute(&mut rows[n]) };
                        cols.diff = clks[1] - clks[0];
                    });

                rows
            })
            .collect::<Vec<_>>()
            .concat();

        // Add final row
        if let Some(last_row) = self.values.last() {
            let mut row = [SC::Val::zero(); NUM_OUTPUT_COLS];
            let cols: &mut OutputCols<SC::Val> = unsafe { transmute(&mut row) };
            cols.is_real = SC::Val::one();
            cols.clk = SC::Val::from_canonical_u32(last_row.0);
            cols.value = SC::Val::from_canonical_u8(last_row.1);
            rows.push(row);
        }

        // TODO: Implement witness data for counter and counter_mult, and then
        // re-enable local_sends and local_receives

        let mut values = rows.concat();
        pad_to_power_of_two::<NUM_OUTPUT_COLS, SC::Val>(&mut values);
        RowMajorMatrix::new(values, NUM_OUTPUT_COLS)
    }

    //fn local_sends(&self) -> Vec<Interaction<SC::Val>> {
    //    let sends = Interaction {
    //        fields: vec![VirtualPairCol::single_main(OUTPUT_COL_MAP.diff)],
    //        count: VirtualPairCol::one(),
    //        argument_index: BusArgument::Local(0),
    //    };
    //    vec![sends]
    //}

    //fn local_receives(&self) -> Vec<Interaction<SC::Val>> {
    //    let receives = Interaction {
    //        fields: vec![VirtualPairCol::single_main(OUTPUT_COL_MAP.counter)],
    //        count: VirtualPairCol::single_main(OUTPUT_COL_MAP.counter_mult),
    //        argument_index: BusArgument::Local(0),
    //    };
    //    vec![receives]
    //}

    fn global_receives(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let opcode = VirtualPairCol::single_main(OUTPUT_COL_MAP.opcode);
        let clk = VirtualPairCol::single_main(OUTPUT_COL_MAP.clk);

        let mut values = (0..CPU_MEMORY_CHANNELS * MEMORY_CELL_BYTES)
            .map(|_| VirtualPairCol::constant(SC::Val::zero()))
            .collect::<Vec<_>>();
        values[MEMORY_CELL_BYTES - 1] = VirtualPairCol::single_main(OUTPUT_COL_MAP.value);

        let mut fields = vec![opcode];
        fields.extend(values);
        fields.push(clk);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(OUTPUT_COL_MAP.is_real),
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

pub trait MachineWithOutputChip<F: Field + Arbitrary + Debug>: MachineWithCpuChip<F> {
    fn output(&self) -> &OutputChip;
    fn output_mut(&mut self) -> &mut OutputChip;
}

instructions!(WriteInstruction);

impl<M, F> Instruction<M, F> for WriteInstruction
where
    M: MachineWithOutputChip<F>,
    F: Field + Arbitrary + Debug,
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
        state
            .output_mut()
            .values
            .push((clk, b.into_iter().last().unwrap()));

        state.cpu_mut().push_bus_op(None, opcode, ops);

        // The immediate value flag should be set, and the immediate operand value should
        // equal zero. We only write one byte of one word at a time to output.
        assert_eq!(ops.is_imm(), 1);
        assert_eq!(ops.c(), 0);
    }
}
