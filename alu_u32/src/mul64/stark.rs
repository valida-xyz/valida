use super::columns::Mul64Cols;
use super::Mul64Chip;

use crate::mul64::columns::NUM_MUL64_COLS;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{PrimeField};

impl<F> BaseAir<F> for Mul64Chip {
    fn width(&self) -> usize {
        NUM_MUL64_COLS
    }
}

impl<F, AB> Air<AB> for Mul64Chip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, _builder: &mut AB) {
        todo!()
    }
}
