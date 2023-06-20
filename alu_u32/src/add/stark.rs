use super::columns::Add32Cols;
use super::{Add32Chip, Add32Opcode, Add32PublicInput};
use core::borrow::Borrow;
use valida_bus::{MachineWithGeneralBus, MachineWithRangeBus};
use valida_machine::{chip, ValidaAirBuilder};

use p3_air::Air;
use p3_field::PrimeField;
use p3_matrix::MatrixRows;

impl<F, M, AB> Air<AB> for Add32Chip
where
    F: PrimeField,
    M: MachineWithGeneralBus<F = F> + MachineWithRangeBus,
    AB: ValidaAirBuilder<F = F, Machine = M, PublicInput = Add32PublicInput<F>>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Add32Cols<AB::Var> = main.row(0).borrow();

        let base = AB::Expr::from(AB::F::from_canonical_u32(1 << 8));

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
            AB::Expr::from(AB::F::from_canonical_u32(Add32Opcode)),
        );

        // TODO: Range check output ([0,256]) using preprocessed lookup table

        chip::eval_permutation_constraints(self, builder);
    }
}
