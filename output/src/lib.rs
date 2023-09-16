use crate::columns::{OutputCols, NUM_OUTPUT_COLS, OUTPUT_COL_MAP};
use core::iter;
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{
    instructions, Chip, Instruction, Interaction, Operands, CPU_MEMORY_CHANNELS, MEMORY_CELL_BYTES,
};
use valida_opcodes::WRITE;

use p3_air::VirtualPairCol;
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;
use valida_util::pad_to_power_of_two;

pub mod columns;
pub mod stark;

#[derive(Default)]
pub struct OutputChip {
    pub values: Vec<(u32, u8)>, // (clk, byte)
}

impl<F, M> Chip<M> for OutputChip
where
    F: PrimeField,
    M: MachineWithGeneralBus<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
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
                for n in 0..num_rows {
                    let mut row = [M::F::ZERO; NUM_OUTPUT_COLS];
                    let cols: &mut OutputCols<M::F> = unsafe { transmute(&mut row) };
                    if n == 0 {
                        cols.is_real = M::F::ONE;
                        cols.clk = M::F::from_canonical_u32(clk_1);
                        cols.value = M::F::from_canonical_u8(val_1);
                    } else {
                        // Dummy output to satisfy range check
                        cols.clk = M::F::from_canonical_u32(clk_1 + table_len * (n + 1) as u32);
                    }
                    rows.push(row);
                }

                // Compute clock diffs
                rows.iter()
                    .map(|row| row[OUTPUT_COL_MAP.clk])
                    .chain(iter::once(M::F::from_canonical_u32(clk_2)))
                    .collect::<Vec<_>>()
                    .windows(2)
                    .enumerate()
                    .for_each(|(n, clks)| {
                        let cols: &mut OutputCols<M::F> = unsafe { transmute(&mut rows[n]) };
                        cols.diff = clks[1] - clks[0];
                    });

                rows
            })
            .collect::<Vec<_>>()
            .concat();

        // Add final row
        if let Some(last_row) = self.values.last() {
            let mut row = [M::F::ZERO; NUM_OUTPUT_COLS];
            let cols: &mut OutputCols<M::F> = unsafe { transmute(&mut row) };
            cols.is_real = M::F::ONE;
            cols.clk = M::F::from_canonical_u32(last_row.0);
            cols.value = M::F::from_canonical_u8(last_row.1);
            rows.push(row);
        }

        // TODO: Implement witness data for counter and counter_mult, and then
        // re-enable local_sends and local_receives

        let mut values = rows.concat();
        pad_to_power_of_two::<NUM_OUTPUT_COLS, F>(&mut values);
        RowMajorMatrix::new(values, NUM_OUTPUT_COLS)
    }

    //fn local_sends(&self) -> Vec<Interaction<M::F>> {
    //    let sends = Interaction {
    //        fields: vec![VirtualPairCol::single_main(OUTPUT_COL_MAP.diff)],
    //        count: VirtualPairCol::one(),
    //        argument_index: BusArgument::Local(0),
    //    };
    //    vec![sends]
    //}

    //fn local_receives(&self) -> Vec<Interaction<M::F>> {
    //    let receives = Interaction {
    //        fields: vec![VirtualPairCol::single_main(OUTPUT_COL_MAP.counter)],
    //        count: VirtualPairCol::single_main(OUTPUT_COL_MAP.counter_mult),
    //        argument_index: BusArgument::Local(0),
    //    };
    //    vec![receives]
    //}

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::single_main(OUTPUT_COL_MAP.opcode);
        let clk = VirtualPairCol::single_main(OUTPUT_COL_MAP.clk);

        let mut values = (0..CPU_MEMORY_CHANNELS * MEMORY_CELL_BYTES)
            .map(|_| VirtualPairCol::constant(M::F::ZERO))
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

pub trait MachineWithOutputChip: MachineWithCpuChip {
    fn output(&self) -> &OutputChip;
    fn output_mut(&mut self) -> &mut OutputChip;
}

instructions!(WriteInstruction);

impl<M> Instruction<M> for WriteInstruction
where
    M: MachineWithOutputChip,
{
    const OPCODE: u32 = WRITE;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        state
            .output_mut()
            .values
            .push((clk, b.into_iter().last().unwrap()));

        state
            .cpu_mut()
            .push_bus_op(None, <Self as Instruction<M>>::OPCODE, ops);

        // The immediate value flag should be set, and the immediate operand value should
        // equal zero. We only write one byte of one word at a time to output.
        assert_eq!(ops.is_imm(), 1);
        assert_eq!(ops.c(), 0);
    }
}
