use crate::RangeCheckerChip;

use p3_air::{Air, AirBuilder};

impl<AB> Air<AB> for RangeCheckerChip
where
    AB: AirBuilder,
{
    fn eval(&self, _builder: &mut AB) {
        // TODO
    }
}
