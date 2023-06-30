#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec;
use valida_machine::{Chip, Machine, PrimeField, Word};

use p3_matrix::dense::RowMajorMatrix;

pub mod columns;
pub mod stark;

#[derive(Default)]
pub struct RangeCheckerChip {
    pub count: BTreeMap<u32, u32>,
    pub range_max: u32,
}

impl<F, M> Chip<M> for RangeCheckerChip
where
    F: PrimeField,
    M: Machine<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let mut col = vec![M::F::ZERO; self.range_max as usize];
        for n in 0..self.range_max {
            if let Some(c) = self.count.get(&n) {
                col[n as usize] = M::F::from_canonical_u32(*c);
            }
        }
        RowMajorMatrix::new(col, 1)
    }
}

pub trait MachineWithRangeChip: Machine {
    fn range(&self) -> &RangeCheckerChip;
    fn range_mut(&mut self) -> &mut RangeCheckerChip;

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
