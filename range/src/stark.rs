use crate::RangeCheckerChip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder};
use p3_field::AbstractField;
use p3_matrix::MatrixRows;

impl<AB> Air<AB> for RangeCheckerChip
where
    AB: AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        // TODO
    }
}
