use crate::columns::{OutputCols, NUM_OUTPUT_COLS, OUTPUT_COL_MAP};
use core::iter;
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, BusArgument, Chip, Instruction, Interaction, Operands, Word};

use p3_air::VirtualPairCol;
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

const WRITE_OPCODE: u32 = 102;

pub mod columns;
pub mod stark;

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
        let rows = self
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
                    let mut cols: &mut OutputCols<M::F> = unsafe { transmute(&mut row) };
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
                    .for_each(|(n, window)| {
                        let mut cols: &mut OutputCols<M::F> = unsafe { transmute(&mut rows[n]) };
                        cols.diff = window[1] - window[0];
                    });

                rows
            })
            .collect::<Vec<_>>()
            .concat();

        // TODO: Add final row

        RowMajorMatrix::new(rows.concat(), NUM_OUTPUT_COLS)
    }

    fn local_sends(&self) -> Vec<Interaction<M::F>> {
        let sends = Interaction {
            fields: vec![VirtualPairCol::single_main(OUTPUT_COL_MAP.diff)],
            count: VirtualPairCol::one(),
            argument_index: BusArgument::Local(0),
        };
        vec![sends]
    }

    fn local_receives(&self) -> Vec<Interaction<M::F>> {
        let receives = Interaction {
            fields: vec![VirtualPairCol::single_main(OUTPUT_COL_MAP.counter)],
            count: VirtualPairCol::single_main(OUTPUT_COL_MAP.counter_mult),
            argument_index: BusArgument::Local(0),
        };
        vec![receives]
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::single_main(OUTPUT_COL_MAP.opcode);
        let clk = VirtualPairCol::single_main(OUTPUT_COL_MAP.clk);
        let value = VirtualPairCol::single_main(OUTPUT_COL_MAP.value);
        // FIXME: We need to insert dummy "zero" fields here (and in the proper order) so that
        // the RLC can be computed correctly
        let fields = vec![opcode, clk, value];

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
    const OPCODE: u32 = WRITE_OPCODE;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        state
            .output_mut()
            .values
            .push((clk, b.into_iter().last().unwrap()));

        // The immediate value flag should be set, and the immediate operand value should
        // equal zero. We only write one byte of one word at a time to output.
        assert_eq!(ops.is_imm(), 1);
        assert_eq!(ops.c(), 0);
    }
}
