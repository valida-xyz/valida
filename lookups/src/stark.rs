use p3_air::{Air, BaseAir};
use p3_matrix::{dense::RowMajorMatrix, Matrix, MatrixRows};
use valida_machine::{StarkConfig, ValidaAirBuilder, __internal::p3_field::AbstractField};

use crate::{LookupChip, LookupTable, LookupType};

impl<L, F> BaseAir<F> for LookupChip<L, F>
where
    F: AbstractField + Sync,
    L: LookupTable<F> + Sync,
{
    fn width(&self) -> usize {
        self.table().width()
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        match self.lookup_type() {
            LookupType::Preprocessed => Some(self.table().to_row_major_matrix()),
            _ => None,
        }
    }
}

// in a pure lookup, there are no constraints to evaluate on the lookup table or the
// column of mutliplicities.
impl<L, AB> Air<AB> for LookupChip<L, AB::F>
where
    L: LookupTable<AB::F> + Sync,
    AB: ValidaAirBuilder,
{
    fn eval(&self, _builder: &mut AB) {}
}
