use super::columns::NativeFieldCols;
use super::NativeFieldChip;
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField};
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for NativeFieldChip {}

impl<F, AB> Air<AB> for NativeFieldChip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &NativeFieldCols<AB::Var> = main.row_slice(0).borrow();

        let base_m = [1 << 24, 1 << 16, 1 << 8, 1].map(AB::Expr::from_canonical_u32);
        let x = local.input_1;
        let y = local.input_2;
        let z = local.output;
        let b = base_m[3].clone() * x[3]
            + base_m[2].clone() * x[2]
            + base_m[1].clone() * x[1]
            + base_m[0].clone() * x[0];
        let c = base_m[3].clone() * y[3]
            + base_m[2].clone() * y[2]
            + base_m[1].clone() * y[1]
            + base_m[0].clone() * y[0];
        let a = base_m[3].clone() * z[3]
            + base_m[2].clone() * z[2]
            + base_m[1].clone() * z[1]
            + base_m[0].clone() * z[0];

        let a_add = b.clone() + c.clone();
        let a_sub = b.clone() - c.clone();
        let a_mul = b.clone() * c.clone();

        builder.when(local.is_add).assert_eq(a.clone(), a_add);
        builder.when(local.is_sub).assert_eq(a.clone(), a_sub);
        builder.when(local.is_mul).assert_eq(a, a_mul);
    }
}
