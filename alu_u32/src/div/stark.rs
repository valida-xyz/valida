use super::Div32Chip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder};
use p3_field::PrimeField;

impl<F, AB> Air<AB> for Div32Chip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        // TODO
    }
}
