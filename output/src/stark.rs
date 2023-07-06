use crate::columns::OutputCols;
use crate::{OutputChip, WRITE_OPCODE};
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder};
use p3_field::PrimeField;
use p3_matrix::MatrixRows;

impl<F, AB> Air<AB> for OutputChip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &OutputCols<AB::Var> = main.row(0).borrow();
        let next: &OutputCols<AB::Var> = main.row(1).borrow();

        // Address should increment by 1
        builder
            .when_transition()
            .assert_eq(local.addr + AB::F::ONE, next.addr);

        // Bus opcode constraint
        builder.assert_eq(
            local.opcode,
            AB::Expr::from(AB::F::from_canonical_u32(WRITE_OPCODE)),
        );
    }
}
