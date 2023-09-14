use super::Div32Chip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField};

impl<F> BaseAir<F> for Div32Chip {}

impl<F, AB> Air<AB> for Div32Chip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        // TODO
    }
}
