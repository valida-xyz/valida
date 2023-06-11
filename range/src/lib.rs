#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use valida_machine::{Chip, Interaction, LookupData, Machine, PrimeField};

use p3_matrix::dense::RowMajorMatrix;

pub mod columns;
pub mod stark;

#[derive(Default)]
pub struct RangeCheckerChip<M: Machine + ?Sized> {
    pub count: BTreeMap<u32, u32>,
    pub range_max: u32,
    lookup_data: Option<LookupData<M::F, M::EF>>,
}

impl<F, M> Chip<M> for RangeCheckerChip<M>
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
    fn range(&self) -> &RangeCheckerChip<Self>;
    fn range_mut(&mut self) -> &mut RangeCheckerChip<Self>;
}
