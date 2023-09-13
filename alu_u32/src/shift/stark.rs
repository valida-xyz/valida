use super::columns::Shift32Cols;
use super::Shift32Chip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for Shift32Chip {}

impl<F, AB> Air<AB> for Shift32Chip
where
    F: AbstractField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Shift32Cols<AB::Var> = main.row_slice(0).borrow();

        let one = AB::Expr::ONE;

        let bit_base = [1, 2, 4, 8, 16, 32, 64, 128].map(AB::Expr::from_canonical_u32);
        let pow_base = [1 << 1, 1 << 2, 1 << 4].map(AB::Expr::from_canonical_u32);

        // Check that input byte decomposition is correct
        let byte_2: AB::Expr = local
            .bits_2
            .into_iter()
            .zip(bit_base)
            .map(|(bit, base)| bit * base)
            .sum();
        builder.assert_eq(local.input_2[3], byte_2.clone());

        for bit in local.bits_2.iter() {
            builder.assert_bool(*bit);
        }

        // Check that the power of two is correct (limited to 2^31)
        let temp_1 = (local.bits_2[0] * pow_base[0].clone())
            * (local.bits_2[1] * pow_base[1].clone())
            * (local.bits_2[2] * pow_base[2].clone());
        builder.assert_eq(local.temp_1, temp_1);
        builder.assert_eq(
            local.power_of_two[0],
            local.temp_1 * (one.clone() - local.bits_2[3]) * (one.clone() - local.bits_2[4]),
        );
        builder.assert_eq(
            local.power_of_two[1],
            local.temp_1 * local.bits_2[3] * (one.clone() - local.bits_2[4]),
        );
        builder.assert_eq(
            local.power_of_two[2],
            local.temp_1 * (one - local.bits_2[3]) * local.bits_2[4],
        );
        builder.assert_eq(
            local.power_of_two[3],
            local.temp_1 * local.bits_2[3] * local.bits_2[4],
        );

        builder.assert_bool(local.is_shl);
        builder.assert_bool(local.is_shr);
        builder.assert_bool(local.is_shl + local.is_shr);
    }
}
