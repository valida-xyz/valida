use super::columns::Add32Cols;
use super::Add32Opcode;
use core::borrow::Borrow;

use p3_air::{Air, PermutationAirBuilder};
use p3_field::PrimeField;
use p3_matrix::Matrix;

#[derive(Default)]
pub struct Add32Stark {}

impl<AB: PermutationAirBuilder<F = B>, B: PrimeField> Air<AB> for Add32Stark {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Add32Cols<AB::Var> = main.row(0).borrow();

        let base = AB::Exp::from(AB::F::from_canonical_u32(1 << 8));

        let carry_0 = local.input_1[3] + local.input_2[3] - local.output[3];
        let carry_1 = local.input_1[2] + local.input_2[2] + carry_0.clone() - local.output[2];
        let carry_2 = local.input_1[1] + local.input_2[1] + carry_1.clone() - local.output[1];
        let carry_3 = local.input_1[0] + local.input_2[0] + carry_2.clone() - local.output[0];

        builder.assert_zero(carry_0.clone() * (base.clone() + carry_0.clone()));
        builder.assert_zero(carry_1.clone() * (base.clone() + carry_1.clone()));
        builder.assert_zero(carry_2.clone() * (base.clone() + carry_2.clone()));
        builder.assert_zero(carry_3.clone() * (base.clone() + carry_3.clone()));

        // Bus opcode constraint
        builder.assert_eq(
            local.opcode,
            AB::Exp::from(AB::F::from_canonical_u32(Add32Opcode)),
        );

        // TODO: Range check output ([0,256]) using preprocessed lookup table
    }
}
