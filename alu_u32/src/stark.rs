use crate::columns::AluU32Cols;
use core::borrow::Borrow;
use core::mem::MaybeUninit;
use itertools::iproduct;
use valida_machine::Word;

use p3_air::{Air, AirBuilder, PermutationAirBuilder};
use p3_field::PrimeField;
use p3_matrix::Matrix;

/// Set of pairwise coprime moduli
const M0: u32 = (1 << 31) - 1;
const M1: u32 = 997;
const M2: u32 = 971;
const M3: u32 = 967;
const M4: u32 = 11;

pub struct ALU32Stark {}

impl<AB: PermutationAirBuilder<F = B>, B: PrimeField> Air<AB> for ALU32Stark {
    fn eval(&self, builder: &mut AB) {
        self.eval_main(builder);
    }
}

impl ALU32Stark {
    fn eval_main<AB: PermutationAirBuilder<F = B>, B: PrimeField>(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &AluU32Cols<AB::Var> = main.row(0).borrow();

        for (n, m) in [M0, M1, M2, M3, M4].into_iter().enumerate() {
            // Limbs modulo the coprime field modulus (assuming first modulus is native)
            let mut base_m: [AB::Exp; 6] = unsafe { MaybeUninit::uninit().assume_init() };
            for i in 0..6 {
                let b = if n == 0 { 1 << 8 * m } else { (1 << 8 * m) % m };
                base_m[i] = AB::Exp::from(AB::F::from_canonical_u32(b));
            }

            // Partially reduced summation of input product limbs
            let pi_m: AB::Exp = pi_m::<AB>(&base_m, local.input_1, local.input_2);

            // Partially reduced summation of output limbs
            let sigma_m = sigma_m::<AB>(&base_m, local.output);

            // Constrain the witnessed output and quotient
            let m_expr = AB::Exp::from(AB::F::from_canonical_u32(m));
            if n == 0 {
                builder.when_transition().assert_zero(pi_m - sigma_m);
            } else {
                builder
                    .when_transition()
                    .assert_eq(pi_m - sigma_m, local.s * m_expr);
            }
        }
    }
}

fn pi_m<AB: PermutationAirBuilder>(
    base: &[AB::Exp; 6],
    input_1: Word<AB::Var>,
    input_2: Word<AB::Var>,
) -> AB::Exp {
    iproduct!(0..4, 0..4)
        .map(|(i, j)| base[i + j].clone() * input_1[i] * input_2[j])
        .sum()
}

fn sigma_m<AB: PermutationAirBuilder>(base: &[AB::Exp; 6], input: [AB::Var; 8]) -> AB::Exp {
    input
        .into_iter()
        .enumerate()
        .map(|(i, x)| base[i].clone() * x)
        .sum()
}
