use super::columns::Sub32Cols;
use super::Sub32Chip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField};
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for Sub32Chip {}

impl<F, AB> Air<AB> for Sub32Chip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Sub32Cols<AB::Var> = main.row_slice(0).borrow();

        let base = AB::Expr::from_canonical_u32(1 << 8);

        let borrow_1 = local.borrow[0];
        let borrow_2 = local.borrow[1];
        let borrow_3 = local.borrow[2];

        builder.assert_eq(
            local.output[3],
            base.clone() * borrow_1 + local.input_1[3] - local.input_2[3],
        );
        builder.assert_eq(
            local.output[2],
            base.clone() * borrow_2 + local.input_1[2] - local.input_2[2] - borrow_1,
        );
        builder.assert_eq(
            local.output[1],
            base.clone() * borrow_3 + local.input_1[1] - local.input_2[1] - borrow_2,
        );
        builder.assert_eq(
            local.output[0],
            local.input_1[0] - local.input_2[0] - borrow_3,
        );

        builder.assert_bool(borrow_1);
        builder.assert_bool(borrow_2);
        builder.assert_bool(borrow_3);
    }
}
