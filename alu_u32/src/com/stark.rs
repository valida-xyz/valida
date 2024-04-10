use super::columns::Com32Cols;
use super::Com32Chip;
use core::borrow::Borrow;

use crate::com::columns::NUM_COM_COLS;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_matrix::MatrixRowSlices;

impl<F: AbstractField> BaseAir<F> for Com32Chip {
    fn width(&self) -> usize {
        NUM_COM_COLS
    }
}

impl<F, AB> Air<AB> for Com32Chip
where
    F: AbstractField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Com32Cols<AB::Var> = main.row_slice(0).borrow();

        // Check if the first two operand values are equal, in case we're doing a conditional branch.
        // (when is_imm == 1, the second read value is guaranteed to be an immediate value)
        builder.assert_eq(
            local.diff,
            local
                .input_1
                .into_iter()
                .zip(local.input_2)
                .map(|(a, b)| (a - b).square())
                .sum::<AB::Expr>(),
        );
        builder.assert_bool(local.not_equal);
        builder.assert_eq(local.not_equal, local.diff * local.diff_inv);
        let equal = AB::Expr::one() - local.not_equal;
        builder.assert_zero(equal * local.diff);

        builder.assert_bool(local.is_ne);
        builder.assert_bool(local.is_eq);
        builder.assert_bool(local.is_ne + local.is_eq);

        builder.assert_eq(
            local.output,
            local.is_ne * local.not_equal + local.is_eq * (AB::Expr::one() - local.not_equal),
        )
    }
}
