use super::columns::Lt32Cols;
use super::Lt32Chip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for Lt32Chip {}

impl<F, AB> Air<AB> for Lt32Chip
where
    F: AbstractField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Lt32Cols<AB::Var> = main.row_slice(0).borrow();

        let base_2 = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512].map(AB::Expr::from_canonical_u32);

        let bit_comp: AB::Expr = local
            .bits
            .into_iter()
            .zip(base_2.iter().cloned())
            .map(|(bit, base)| bit * base)
            .sum();

        // Check bit decomposition of z = 256 + input_1[n] - input_2[n], where
        // n is the most significant byte that differs between inputs
        for i in 0..3 {
            builder
                .when_ne(local.byte_flag[i], AB::Expr::ONE)
                .assert_eq(local.input_1[i], local.input_2[i]);

            builder.when(local.byte_flag[i]).assert_eq(
                AB::Expr::from_canonical_u32(256) + local.input_1[i] - local.input_2[i],
                bit_comp.clone(),
            );

            builder.assert_bool(local.byte_flag[i]);
        }

        // Check final byte (if no other byte flags were set)
        let flag_sum = local.byte_flag[0] + local.byte_flag[1] + local.byte_flag[2];
        builder.assert_bool(flag_sum.clone());
        builder
            .when_ne(local.multiplicity, AB::Expr::ZERO)
            .when_ne(flag_sum, AB::Expr::ONE)
            .assert_eq(
                AB::Expr::from_canonical_u32(256) + local.input_1[3] - local.input_2[3],
                bit_comp.clone(),
            );

        // Output constraints
        builder.when(local.bits[8]).assert_zero(local.output);
        builder
            .when_ne(local.multiplicity, AB::Expr::ZERO)
            .when_ne(local.bits[8], AB::Expr::ONE)
            .assert_one(local.output);

        // Check bit decomposition
        for bit in local.bits.into_iter() {
            builder.assert_bool(bit);
        }
    }
}
