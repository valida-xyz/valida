use super::columns::Div32Cols;
use super::Div32Chip;
use core::borrow::Borrow;
use itertools::iproduct;
use valida_machine::Word;

use p3_air::{Air, AirBuilder};
use p3_field::{AbstractField, PrimeField};
use p3_matrix::MatrixRows;

impl<F, AB> Air<AB> for Div32Chip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Div32Cols<AB::Var> = main.row(0).borrow();
        let next: &Div32Cols<AB::Var> = main.row(1).borrow();

        // output = input_1 / input_2
        // therefore, by rearranging, input_1 = output * input_2 and we can apply the
        // same congruence checks as in the multiplication case.

        // Limb weights modulo 2^32
        let base_m = [1 << 24, 1 << 16, 1 << 8, 1].map(AB::Expr::from_canonical_u32);

        // Partially reduced summation of input product limbs (mod 2^32)
        let pi = pi_m::<4, AB>(&base_m, local.output, local.input_2);

        // Partially reduced summation of output limbs (mod 2^32)
        let sigma = sigma_m::<4, AB>(&base_m, local.input_1);

        // Partially reduced summation of input product limbs (mod 2^16)
        let pi_prime = pi_m::<2, AB>(&base_m[..2], local.output, local.input_2);

        // Partially reduced summation of output limbs (mod 2^16)
        let sigma_prime = sigma_m::<2, AB>(&base_m[..2], local.input_1);

        // Congruence checks
        builder.assert_eq(pi - sigma, local.r * AB::Expr::TWO);
        builder.assert_eq(pi_prime - sigma_prime, local.s * base_m[1].clone());

        // Range check counter
        builder
            .when_first_row()
            .assert_eq(local.counter, AB::Expr::ONE);
        builder.when_transition().assert_zero(
            (local.counter - next.counter) * (local.counter + AB::Expr::ONE - next.counter),
        );
        builder
            .when_last_row()
            .assert_eq(local.counter, AB::Expr::from_canonical_u32(1 << 10));
    }
}

// HELPER FUNCTIONS
// -------------------------------------------------------------------------------------------------

fn pi_m<const N: usize, AB: AirBuilder>(
    base: &[AB::Expr],
    input_1: Word<AB::Var>,
    input_2: Word<AB::Var>,
) -> AB::Expr {
    iproduct!(0..N, 0..N)
        .filter(|(i, j)| i + j < N)
        .map(|(i, j)| base[i + j].clone() * input_1[i] * input_2[j])
        .sum()
}

fn sigma_m<const N: usize, AB: AirBuilder>(base: &[AB::Expr], input: Word<AB::Var>) -> AB::Expr {
    input
        .into_iter()
        .enumerate()
        .map(|(i, x)| base[i].clone() * x)
        .sum()
}