#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec;
use valida_machine::{Chip, Machine, PrimeField, Word};

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
    M: Machine<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let mut col = vec![M::F::ZERO; MAX as usize];
        for n in 0..MAX {
            if let Some(c) = self.count.get(&n) {
                col[n as usize] = M::F::from_canonical_u32(*c);
            }
        }
        RowMajorMatrix::new(col, 1)
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
