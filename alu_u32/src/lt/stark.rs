use super::columns::Lt32Cols;
use super::Lt32Chip;
use core::borrow::Borrow;

use crate::lt::columns::NUM_LT_COLS;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for Lt32Chip {
    fn width(&self) -> usize {
        NUM_LT_COLS
    }
}

impl<F, AB> Air<AB> for Lt32Chip
where
    F: AbstractField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Lt32Cols<AB::Var> = main.row_slice(0).borrow();

        let base_2 = [1, 2, 4, 8, 16, 32, 64, 128, 256].map(AB::Expr::from_canonical_u32);

        let bit_comp: AB::Expr = local
            .bits
            .into_iter()
            .zip(base_2.iter().cloned())
            .map(|(bit, base)| bit * base)
            .sum();

        // check that the n-th byte flag is set, where n is the first byte that differs between the two inputs

        // ensure at most one byte flag is set
        let flag_sum =
            local.byte_flag[0] + local.byte_flag[1] + local.byte_flag[2] + local.byte_flag[3];
        builder.assert_bool(flag_sum.clone());
        // check that bytes before the first set byte flag are all equal
        // case: top bytes match
        builder
            .when_ne(local.byte_flag[0], AB::Expr::one())
            .assert_eq(local.input_1[0], local.input_2[0]);
        // case: top two bytes match
        builder
            .when_ne(local.byte_flag[0] + local.byte_flag[1], AB::Expr::one())
            .assert_eq(local.input_1[1], local.input_2[1]);
        // case: top three bytes match
        builder
            .when_ne(
                local.byte_flag[0] + local.byte_flag[1] + local.byte_flag[2],
                AB::Expr::one(),
            )
            .assert_eq(local.input_1[2], local.input_2[2]);
        // case: top four bytes match; must set z = 0
        builder
            .when_ne(flag_sum.clone(), AB::Expr::one())
            .assert_eq(local.input_1[3], local.input_2[3]);
        builder
            .when_ne(flag_sum.clone(), AB::Expr::one())
            .assert_eq(bit_comp.clone(), AB::Expr::zero());

        // Check bit decomposition of z = 256 + input_1[n] - input_2[n]
        // when `n` is the first byte that differs between the two inputs.
        for i in 0..4 {
            builder.when(local.byte_flag[i]).assert_eq(
                AB::Expr::from_canonical_u32(256) + local.input_1[i] - local.input_2[i],
                bit_comp.clone(),
            );
            // ensure that when the n-th byte flag is set, the n-th bytes are actually different
            builder.when(local.byte_flag[i]).assert_eq(
                (local.input_1[i] - local.input_2[i]) * local.diff_inv,
                AB::Expr::one(),
            );
            builder.assert_bool(local.byte_flag[i]);
        }

        // Check the bit decomposition of the top bytes:
        let top_comp_1: AB::Expr = local
            .top_bits_1
            .into_iter()
            .zip(base_2.iter().cloned())
            .map(|(bit, base)| bit * base)
            .sum();
        let top_comp_2: AB::Expr = local
            .top_bits_2
            .into_iter()
            .zip(base_2.iter().cloned())
            .map(|(bit, base)| bit * base)
            .sum();
        builder.assert_eq(top_comp_1, local.input_1[0]);
        builder.assert_eq(top_comp_2, local.input_2[0]);

        let is_signed = local.is_slt + local.is_sle;
        let is_unsigned = AB::Expr::one() - is_signed.clone();
        let same_sign = AB::Expr::one() - local.different_signs;
        let are_equal = AB::Expr::one() - flag_sum.clone();

        builder
            .when(is_unsigned.clone())
            .assert_zero(local.different_signs);

        // Check that `different_signs` is set correctly by comparing sign bits.
        builder
            .when(is_signed.clone())
            .when_ne(local.top_bits_1[7], local.top_bits_2[7])
            .assert_eq(local.different_signs, AB::Expr::one());
        builder
            .when(local.different_signs)
            .assert_eq(local.byte_flag[0], AB::Expr::one());
        // local.top_bits_1[7] and local.top_bits_2[7] are boolean; their sum is 1 iff they are unequal.
        builder
            .when(local.different_signs)
            .assert_eq(local.top_bits_1[7] + local.top_bits_2[7], AB::Expr::one());

        builder.assert_bool(local.is_lt);
        builder.assert_bool(local.is_lte);
        builder.assert_bool(local.is_slt);
        builder.assert_bool(local.is_sle);
        builder.assert_bool(local.is_lt + local.is_lte + local.is_slt + local.is_sle);

        // Output constraints
        // Case 0: input_1 > input_2 as unsigned ints; equivalently, local.bits[8] == 1
        //  when both inputs have the same sign, signed and unsigned inequality agree.
        builder
            .when(local.bits[8])
            .when(is_unsigned.clone() + same_sign.clone())
            .assert_zero(local.output);
        // when the inputs have different signs, signed inequality is the opposite of unsigned inequality.
        builder
            .when(local.bits[8])
            .when(local.different_signs)
            .assert_one(local.output);

        // Case 1: input_1 < input_2 as unsigned ints; equivalently, local.bits[8] == is_equal == 0.
        builder
            // when are_equal == 1, we have already enforced that local.bits[8] == 0
            .when_ne(local.bits[8] + are_equal.clone(), AB::Expr::one())
            .when(is_unsigned.clone() + same_sign.clone())
            .assert_one(local.output);
        builder
            .when_ne(local.bits[8] + are_equal.clone(), AB::Expr::one())
            .when(local.different_signs)
            .assert_zero(local.output);

        // Case 2: input_1 == input_2; equivalently, are_equal == 1
        // output should be 1 if is_lte or is_sle
        builder
            .when(are_equal.clone())
            .when(local.is_lte + local.is_sle)
            .assert_one(local.output);
        // output should be 0 if is_lt or is_slt
        builder
            .when(are_equal.clone())
            .when(local.is_lt + local.is_slt)
            .assert_zero(local.output);

        // Check "bit" values are all boolean
        for bit in local
            .bits
            .into_iter()
            .chain(local.top_bits_1.into_iter())
            .chain(local.top_bits_2.into_iter())
        {
            builder.assert_bool(bit);
        }
    }
}
