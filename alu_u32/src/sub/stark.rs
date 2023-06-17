use super::columns::Sub32Cols;
use crate::Sub32Opcode;
use core::borrow::Borrow;
use valida_machine::{Machine, ValidaAir};

use p3_air::{Air, PermutationAirBuilder};
use p3_field::PrimeField;
use p3_matrix::Matrix;

pub struct Sub32Stark {}

impl<M: Machine, AB: PermutationAirBuilder<F = B>, B: PrimeField> ValidaAir<AB, M> for Sub32Stark {
    fn eval(&self, builder: &mut AB, machine: &M) {
        let main = builder.main();
        let local: &Sub32Cols<AB::Var> = main.row(0).borrow();

        let base = AB::Expr::from(AB::F::from_canonical_u32(1 << 8));

        let sub_0 = local.input_1[3] - local.input_2[3];
        let sub_1 = local.input_1[2] - local.input_2[2];
        let sub_2 = local.input_1[1] - local.input_2[1];
        let sub_3 = local.input_1[0] - local.input_2[0];

        let borrow_0 = sub_0.clone() - local.output[3];
        let borrow_1 = sub_1.clone() - local.output[2];
        let borrow_2 = sub_2.clone() - local.output[1];
        let borrow_3 = sub_3.clone() - local.output[0];

        // First byte
        builder.assert_zero(borrow_0.clone() * (base.clone() - sub_0 - local.output[3]));
        builder
            .assert_zero(borrow_0 * (sub_1.clone() - local.output[2] - AB::Expr::from(AB::F::ONE)));

        // Second byte
        builder.assert_zero(borrow_1.clone() * (base.clone() - sub_1 - local.output[2]));
        builder
            .assert_zero(borrow_1 * (sub_2.clone() - local.output[1] - AB::Expr::from(AB::F::ONE)));

        // Third byte
        builder.assert_zero(borrow_2.clone() * (base.clone() - sub_2 - local.output[1]));
        builder.assert_zero(borrow_2 * (sub_3 - local.output[0] - AB::Expr::from(AB::F::ONE)));

        // Bus opcode constraint
        builder.assert_eq(
            local.opcode,
            AB::Expr::from(AB::F::from_canonical_u32(Sub32Opcode)),
        );

        todo!()
    }
}
