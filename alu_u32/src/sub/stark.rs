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

        let sub_0 = local.input_1[3] - local.input_2[3];
        let sub_1 = local.input_1[2] - local.input_2[2];
        let sub_2 = local.input_1[1] - local.input_2[1];
        let sub_3 = local.input_1[0] - local.input_2[0];

        let borrow_0 = sub_0.clone() - local.output[3];
        let borrow_1 = sub_1.clone() - local.output[2];
        let borrow_2 = sub_2.clone() - local.output[1];
        let _borrow_3 = sub_3.clone() - local.output[0];

        // First byte
        builder.assert_zero(borrow_0.clone() * (base.clone() - sub_0 - local.output[3]));
        builder.assert_zero(borrow_0 * (sub_1.clone() - local.output[2] - AB::Expr::ONE));

        // Second byte
        builder.assert_zero(borrow_1.clone() * (base.clone() - sub_1 - local.output[2]));
        builder.assert_zero(borrow_1 * (sub_2.clone() - local.output[1] - AB::Expr::ONE));

        // Third byte
        builder.assert_zero(borrow_2.clone() * (base.clone() - sub_2 - local.output[1]));
        builder.assert_zero(borrow_2 * (sub_3 - local.output[0] - AB::Expr::ONE));

        // TODO: unfinished?
    }
}
