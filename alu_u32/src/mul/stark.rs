use super::columns::Mul32Cols;
use super::{Mul32Chip, Mul32Opcode, Mul32PublicInput};
use core::borrow::Borrow;
use core::mem::MaybeUninit;
use itertools::iproduct;
use valida_bus::MachineWithGeneralBus;
use valida_machine::{chip, ValidaAirBuilder, Word};

use p3_air::{Air, AirBuilder, PermutationAirBuilder};
use p3_field::PrimeField;
use p3_matrix::MatrixRows;

impl<F, M, AB> Air<AB> for Mul32Chip
where
    F: PrimeField,
    M: MachineWithGeneralBus<F = F>,
    AB: ValidaAirBuilder<F = F, Machine = M, PublicInput = Mul32PublicInput<F>>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Mul32Cols<AB::Var> = main.row(0).borrow();
        let next: &Mul32Cols<AB::Var> = main.row(1).borrow();

        // Limb weights modulo 2^32
        let mut base_m: [AB::Expr; 4] = unsafe { MaybeUninit::uninit().assume_init() };
        for (i, b) in [1 << 24, 1 << 16, 1 << 8, 1].into_iter().enumerate() {
            base_m[i] = AB::Expr::from(AB::F::from_canonical_u32(b));
        }

        // Partially reduced summation of input product limbs (mod 2^32)
        let pi = pi_m::<4, AB>(&base_m, local.input_1, local.input_2);

        // Partially reduced summation of output limbs (mod 2^32)
        let sigma = sigma_m::<4, AB>(&base_m, local.output);

        // Partially reduced summation of input product limbs (mod 2^16)
        let pi_prime = pi_m::<2, AB>(&base_m[..2], local.input_1, local.input_2);

        // Partially reduced summation of output limbs (mod 2^16)
        let sigma_prime = sigma_m::<2, AB>(&base_m[..2], local.output);

        // Congruence checks
        builder.assert_eq(pi - sigma, local.r * AB::Expr::from(AB::F::TWO));
        builder.assert_eq(pi_prime - sigma_prime, local.s * base_m[1].clone());

        // Range check counter
        builder
            .when_first_row()
            .assert_eq(local.counter, AB::Expr::from(AB::F::ONE));
        builder.when_transition().assert_zero(
            (local.counter - next.counter)
                * (local.counter + AB::Expr::from(AB::F::ONE) - next.counter),
        );
        builder.when_last_row().assert_eq(
            local.counter,
            AB::Expr::from(AB::F::from_canonical_u32(1 << 10)),
        );

        // Bus opcode constraint
        builder.assert_eq(
            local.opcode,
            AB::Expr::from(AB::F::from_canonical_u32(Mul32Opcode)),
        );

        chip::eval_permutation_constraints(self, builder);
    }
}

fn pi_m<const N: usize, AB: PermutationAirBuilder>(
    base: &[AB::Expr],
    input_1: Word<AB::Var>,
    input_2: Word<AB::Var>,
) -> AB::Expr {
    iproduct!(0..N, 0..N)
        .filter(|(i, j)| i + j < N)
        .map(|(i, j)| base[i + j].clone() * input_1[i] * input_2[j])
        .sum()
}

fn sigma_m<const N: usize, AB: PermutationAirBuilder>(
    base: &[AB::Expr],
    input: Word<AB::Var>,
) -> AB::Expr {
    input
        .into_iter()
        .enumerate()
        .map(|(i, x)| base[i].clone() * x)
        .sum()
}
