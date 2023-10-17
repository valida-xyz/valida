use super::Div32Chip;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::PrimeField;

impl<F> BaseAir<F> for Div32Chip {}

impl<F, AB> Air<AB> for Div32Chip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, _builder: &mut AB) {
        // TODO
    }
}
