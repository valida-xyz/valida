use crate::columns::{StaticDataCols, NUM_STATIC_DATA_COLS};
use crate::StaticDataChip;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;

impl<F> BaseAir<F> for StaticDataChip {
    fn width(&self) -> usize {
        NUM_STATIC_DATA_COLS
    }
}

impl<AB> Air<AB> for StaticDataChip
where
    AB: AirBuilder,
{
    fn eval(&self, builder: &mut AB) {
        self.eval_main(builder);
    }
}

impl StaticDataChip {
    fn eval_main<AB: AirBuilder>(&self, builder: &mut AB) {
        // TODO
    }
}
