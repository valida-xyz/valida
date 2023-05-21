use super::columns::Add32Cols;
use core::borrow::Borrow;
use itertools::iproduct;
use valida_machine::Word;

use p3_air::{Air, AirBuilder, PermutationAirBuilder};
use p3_field::PrimeField;
use p3_matrix::Matrix;

pub struct Add32Stark {}

impl<AB: PermutationAirBuilder<F = B>, B: PrimeField> Air<AB> for Add32Stark {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Add32Cols<AB::Var> = main.row(0).borrow();

        let base = AB::Exp::from(AB::F::from_canonical_u32(1 << 8));

        let carry_0 = local.input_1[3] + local.input_2[3] - local.output[3];
        let carry_1 = local.input_1[2] + local.input_2[2] + carry_0.clone() - local.output[1];
        let carry_2 = local.input_1[1] + local.input_2[1] + carry_1.clone() - local.output[2];
        let add_overflow_3 = local.input_1[0] + local.input_2[0] + carry_2.clone();

        builder
            .when_transition()
            .assert_zero(carry_0 * (base.clone() - local.output[3]));

        builder
            .when_transition()
            .assert_zero(carry_1 * (base.clone() - local.output[2]));

        builder
            .when_transition()
            .assert_zero(carry_2 * (base.clone() - local.output[1]));

        builder
            .when_transition()
            .assert_zero(add_overflow_3 - local.output[0]);
    }
}
