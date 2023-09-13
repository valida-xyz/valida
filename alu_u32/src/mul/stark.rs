use super::columns::Mul32Cols;
use super::Mul32Chip;
use core::borrow::Borrow;
use itertools::iproduct;
use valida_machine::Word;

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
        let base_m = [1, 1 << 8, 1 << 16, 1 << 24].map(AB::Expr::from_canonical_u32);

        // Partially reduced summation of input product limbs (mod 2^32)
        let pi = pi_m::<4, AB>(&base_m, local.input_1, local.input_2);

        // Partially reduced summation of output limbs (mod 2^32)
        let sigma = sigma_m::<4, AB>(&base_m, local.output);

        // Partially reduced summation of input product limbs (mod 2^16)
        let pi_prime = pi_m::<2, AB>(&base_m[..2], local.input_1, local.input_2);

        // Partially reduced summation of output limbs (mod 2^16)
        let sigma_prime = sigma_m::<2, AB>(&base_m[..2], local.output);

        // Congruence checks
        builder.assert_eq(pi - sigma, local.r * AB::Expr::TWO);
        builder.assert_eq(pi_prime - sigma_prime, local.s * base_m[2].clone());

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

fn pi_m<const N: usize, AB: AirBuilder>(
    base: &[AB::Expr],
    input_1: Word<AB::Var>,
    input_2: Word<AB::Var>,
) -> AB::Expr {
    iproduct!(0..N, 0..N)
        .filter(|(i, j)| i + j < N)
        .map(|(i, j)| base[i + j].clone() * input_1[3 - i] * input_2[3 - j])
        .sum()
}

fn sigma_m<const N: usize, AB: AirBuilder>(base: &[AB::Expr], input: Word<AB::Var>) -> AB::Expr {
    input
        .into_iter()
        .rev()
        .take(N)
        .enumerate()
        .map(|(i, x)| base[i].clone() * x)
        .sum()
}
