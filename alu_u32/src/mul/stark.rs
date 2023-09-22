use super::columns::Mul32Cols;
use super::{pi_m, sigma_m, Mul32Chip};
use core::borrow::Borrow;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField};
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for Mul32Chip {}

impl<F, AB> Air<AB> for Mul32Chip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Mul32Cols<AB::Var> = main.row_slice(0).borrow();
        let next: &Mul32Cols<AB::Var> = main.row_slice(1).borrow();

        // Limb weights modulo 2^32
        let base_m32 = [1, 1 << 8, 1 << 16, 1 << 24].map(AB::Expr::from_canonical_u32);

        // Limb weights modulo 2^16
        let base_m16 = [1, 1 << 8].map(AB::Expr::from_canonical_u32);

        // Partially reduced summation of input product limbs (mod 2^32)
        let pi = pi_m::<4, AB::Var, AB::Expr>(&base_m32, local.input_1, local.input_2);

        // Partially reduced summation of output limbs (mod 2^32)
        let sigma = sigma_m::<4, AB::Var, AB::Expr>(&base_m32, local.output);

        // Partially reduced summation of input product limbs (mod 2^16)
        let pi_prime = pi_m::<2, AB::Var, AB::Expr>(&base_m16, local.input_1, local.input_2);

        // Partially reduced summation of output limbs (mod 2^16)
        let sigma_prime = sigma_m::<2, AB::Var, AB::Expr>(&base_m16, local.output);

        // Congruence checks
        builder.assert_eq(pi - sigma, local.r * AB::Expr::from_wrapped_u64(1 << 32));
        builder.assert_eq(pi_prime - sigma_prime, local.s * base_m32[2].clone());

        // Range check counter
        builder
            .when_first_row()
            .assert_eq(local.counter, AB::Expr::ONE);
        let counter_diff = next.counter - local.counter;
        builder
            .when_transition()
            .assert_zero(counter_diff.clone() * (counter_diff - AB::Expr::ONE));
        builder
            .when_last_row()
            .assert_eq(local.counter, AB::Expr::from_canonical_u32(1 << 10));
    }
}
