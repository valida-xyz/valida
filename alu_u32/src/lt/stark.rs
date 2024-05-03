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

        let base_2 = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512].map(AB::Expr::from_canonical_u32);

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

        builder.assert_bool(local.is_lt);
        builder.assert_bool(local.is_lte);
        builder.assert_bool(local.is_lt + local.is_lte);

        // Output constraints
        // local.bits[8] is 1 iff input_1 > input_2: output should be 0
        builder.when(local.bits[8]).assert_zero(local.output);
        // output should be 1 if is_lte & input_1 == input_2
        builder
            .when(local.is_lte)
            .when_ne(flag_sum.clone(), AB::Expr::one())
            .assert_one(local.output);
        // output should be 0 if is_lt & input_1 == input_2
        builder
            .when(local.is_lt)
            .when_ne(flag_sum, AB::Expr::one())
            .assert_zero(local.output);

        // Check bit decomposition
        for bit in local.bits.into_iter() {
            builder.assert_bool(bit);
        }
    }
}
