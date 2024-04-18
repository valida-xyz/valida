use crate::columns::{NUM_STATIC_DATA_PREPROCESSED_COLS, NUM_STATIC_DATA_COLS};
use crate::StaticDataChip;

use alloc::vec::Vec;
use alloc::vec;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_matrix::dense::RowMajorMatrix;

impl<F: AbstractField> BaseAir<F> for StaticDataChip {
    fn width(&self) -> usize {
        NUM_STATIC_DATA_COLS
    }

    fn preprocessed_trace(&self) -> RowMajorMatrix<F> {
        let mut rows = self.cells.iter()
            .map(|(addr, value)| {
                let mut row: Vec<F> = vec![F::from_canonical_u32(*addr)];
                row.extend(value.0.into_iter().map(F::from_canonical_u8).collect::<Vec<_>>());
                row.push(F::one());
                row
            })
            .flatten()
            .collect::<Vec<_>>();
        rows.resize(rows.len().next_power_of_two() * NUM_STATIC_DATA_PREPROCESSED_COLS, F::zero());
        RowMajorMatrix::new(rows, NUM_STATIC_DATA_PREPROCESSED_COLS)
    }
}

impl<AB> Air<AB> for StaticDataChip
where
    AB: AirBuilder,
{
    fn eval(&self, _builder: &mut AB) {
        // TODO: check equality of main trace with preprocessed trace
    }
}
