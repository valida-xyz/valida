use super::columns::Add32Cols;
use super::Add32Chip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::PrimeField;
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for Add32Chip {}

impl<F, AB> Air<AB> for Add32Chip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Add32Cols<AB::Var> = main.row_slice(0).borrow();

        let one = AB::F::ONE;
        let base = AB::F::from_canonical_u32(1 << 8);

        let carry_1 = local.carry[0];
        let carry_2 = local.carry[1];
        let carry_3 = local.carry[2];

        let overflow_0 = local.input_1[3] + local.input_2[3] - local.output[3];
        let overflow_1 = local.input_1[2] + local.input_2[2] - local.output[2] + carry_1;
        let overflow_2 = local.input_1[1] + local.input_2[1] - local.output[1] + carry_2;
        let overflow_3 = local.input_1[0] + local.input_2[0] - local.output[0] + carry_3;

        // Limb constraints
        builder.assert_zero(overflow_0.clone() * (overflow_0.clone() - base.clone()));
        builder.assert_zero(overflow_1.clone() * (overflow_1.clone() - base.clone()));
        builder.assert_zero(overflow_2.clone() * (overflow_2.clone() - base.clone()));
        builder.assert_zero(overflow_3.clone() * (overflow_3 - base.clone()));

        // Carry constraints
        builder.assert_zero(
            overflow_0.clone() * (carry_1 - one) + (overflow_0 - base.clone()) * carry_1,
        );
        builder.assert_zero(
            overflow_1.clone() * (carry_2 - one) + (overflow_1 - base.clone()) * carry_2,
        );
        builder.assert_zero(overflow_2.clone() * (carry_3 - one) + (overflow_2 - base) * carry_3);
    }
}
