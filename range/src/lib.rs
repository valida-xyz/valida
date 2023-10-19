#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use columns::{RangeCols, NUM_RANGE_COLS, RANGE_COL_MAP};
use core::mem::transmute;
use valida_bus::MachineWithRangeBus8;
use valida_machine::Interaction;
use valida_machine::{Chip, Machine, PrimeField, Word};

use p3_air::VirtualPairCol;
use p3_matrix::dense::RowMajorMatrix;

pub mod columns;
pub mod stark;

#[derive(Default)]
pub struct RangeCheckerChip<const MAX: u32> {
    pub count: BTreeMap<u32, u32>,
}

impl<F, M, const MAX: u32> Chip<M> for RangeCheckerChip<MAX>
where
    F: PrimeField,
    M: MachineWithRangeBus8<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let mut rows = vec![[F::zero(); NUM_RANGE_COLS]; MAX as usize];
        for (n, row) in rows.iter_mut().enumerate() {
            let cols: &mut RangeCols<F> = unsafe { transmute(row) };
            // FIXME: This is very inefficient when the range is large.
            // Iterate over key/val pairs instead in a separate loop.
            if let Some(c) = self.count.get(&(n as u32)) {
                cols.mult = M::F::from_canonical_u32(*c);
            }
            cols.counter = M::F::from_canonical_u32(n as u32);
        }
        RowMajorMatrix::new(rows.concat(), NUM_RANGE_COLS)
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let input = VirtualPairCol::single_main(RANGE_COL_MAP.counter);

        let receive = Interaction {
            fields: vec![input],
            count: VirtualPairCol::single_main(RANGE_COL_MAP.mult),
            argument_index: machine.range_bus(),
        };
        vec![receive]
    }
}

pub trait MachineWithRangeChip<const MAX: u32>: Machine {
    fn range(&self) -> &RangeCheckerChip<MAX>;
    fn range_mut(&mut self) -> &mut RangeCheckerChip<MAX>;

    /// Record the components of the word in the range check counter
    fn range_check<I: Into<u32>>(&mut self, value: Word<I>) {
        for v in value {
            self.range_mut()
                .count
                .entry(v.into())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
    }
}
