use super::Div32Chip;

use crate::div::columns::NUM_DIV_COLS;
use p3_air::{Air, AirBuilder, BaseAir};

impl<F> BaseAir<F> for Div32Chip {
    fn width(&self) -> usize {
        NUM_DIV_COLS
    }
}

impl<F, AB> Air<AB> for Div32Chip
where
    AB: AirBuilder<F = F>,
{
    fn eval(&self, _builder: &mut AB) {
        // TODO
    }
}
