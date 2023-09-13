use super::columns::Bitwise32Cols;
use super::Bitwise32Chip;
use core::borrow::Borrow;
use valida_machine::MEMORY_CELL_BYTES;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for Bitwise32Chip {}

impl<F, AB> Air<AB> for Bitwise32Chip
where
    F: AbstractField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Bitwise32Cols<AB::Var> = main.row_slice(0).borrow();

        let base_2 = [1, 2, 4, 8, 16, 32, 64, 128].map(AB::Expr::from_canonical_u32);

        for i in 0..MEMORY_CELL_BYTES {
            let byte_1: AB::Expr = local.bits_1[i]
                .into_iter()
                .zip(base_2.iter().cloned())
                .map(|(bit, base)| bit * base)
                .sum();
            let byte_2: AB::Expr = local.bits_2[i]
                .into_iter()
                .zip(base_2.iter().cloned())
                .map(|(bit, base)| bit * base)
                .sum();

            // Check that input byte decomposition is correct
            builder.assert_eq(local.input_1[i], byte_1.clone());
            builder.assert_eq(local.input_2[i], byte_2.clone());

            let bitwise_and: AB::Expr = local.bits_1[i]
                .into_iter()
                .zip(local.bits_2[i])
                .zip(base_2.iter().cloned())
                .map(|((bit_1, bit_2), base)| bit_1 * bit_2 * base)
                .sum();
            let bitwise_or: AB::Expr = byte_1.clone() + byte_2.clone() - bitwise_and.clone();
            let bitwise_xor: AB::Expr = byte_1 + byte_2 - AB::Expr::TWO * bitwise_and.clone();

            // Check the resulting output byte
            builder
                .when(local.is_and)
                .assert_eq(bitwise_and.clone(), local.output[i]);
            builder
                .when(local.is_or)
                .assert_eq(bitwise_or.clone(), local.output[i]);
            builder
                .when(local.is_xor)
                .assert_eq(bitwise_xor.clone(), local.output[i]);

            // Check that bits are boolean values
            for bit in local.bits_1[i].into_iter().chain(local.bits_2[i]) {
                builder.assert_bool(bit);
            }
        }

        builder.assert_bool(local.is_and);
        builder.assert_bool(local.is_or);
        builder.assert_bool(local.is_xor);
        builder.assert_bool(local.is_and + local.is_or + local.is_xor);
    }
}
