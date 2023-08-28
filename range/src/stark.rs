use crate::RangeCheckerChip;

use p3_air::{Air, AirBuilder};

impl<AB, const MAX: u32> Air<AB> for RangeCheckerChip<MAX>
where
    AB: AirBuilder,
{
    fn eval(&self, _builder: &mut AB) {
        // TODO
    }
}
