use super::columns::{Bitwise32Cols, NUM_COLS};
use super::Bitwise32Chip;
use core::borrow::Borrow;
use valida_machine::MEMORY_CELL_BYTES;

use p3_air::{Air, AirBuilder, BaseAir, PermutationAirBuilder};
use p3_field::AbstractField;
use p3_matrix::MatrixRowSlices;

impl<F> BaseAir<F> for Bitwise32Chip {}

/// Commits the bitwise op (i1, i2, o1) with a base-256 encoding where i1, i2, o1 \in [0, 8).
/// Assumes the field is large enough to encode the result (~2^25).
#[inline]
fn commit_bitwise_op<F: AbstractField>(i1: F, i2: F, o1: F) -> F {
    let b1 = F::from_canonical_usize(1);
    let b2 = F::from_canonical_usize(1 << 8);
    let b3 = F::from_canonical_usize(1 << 16);
    i1 * b1 + i2 * b2 + o1 * b3
}

impl<F, AB> Air<AB> for Bitwise32Chip
where
    F: AbstractField,
    AB: PermutationAirBuilder<F = F>,
{
    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        // The trace layout.
        const BYTE_RANGE_CHECK_COL: usize = 0;
        const BITWISE_AND_COL: usize = 1;
        const BITWISE_OR_COL: usize = 2;
        const BITWISE_XOR_COL: usize = 3;
        const NUM_PREPROCESSED_COLS: usize = 4;;
        const NUM_ROWS: usize = 1 << 16;

        let rows: Vec<[F; NUM_PREPROCESSED_COLS]> = Vec::new();
        for i in nb_rows {
            // Initialize a row with zeros.
            let row = [F::ZERO; NUM_PREPROCESSED_COLS];

            // Set the byte range check column.
            row[BYTE_RANGE_CHECK_COL] = if i < 8 {
                F::from_canonical_usize(i)
            } else {
                F::from_canonical_usize(7)
            };

            // Calculate the input bytes for the bitwise ops.
            let i1 = i % 256;
            let i2 = i / 256;

            // Set the and lookup column.
            let and = i1 & i2;
            row[BITWISE_AND_COL] = commit_bitwise_op(i1, i2, and);

            // Set the or lookup column.
            let or = i1 | i2;
            row[BITWISE_OR_COL] = commit_bitwise_op(i1, i2, or);

            // Set the xor lookup column.
            let xor = i1 ^ i2;
            row[BITWISE_XOR_COL] = commit_bitwise_op(i1, i2, xor);
        }

        Some(RowMajorMatrix::new(
            rows.into_iter().flatten().collect::<Vec<_>>(),
            NUM_PREPROCESSED_COLS,
        ))
    }

    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &Bitwise32Cols<AB::Var> = main.row_slice(0).borrow();

        for i in 0..MEMORY_CELL_BYTES {
            // TODO: add lookups
            builder
                .when(local.is_and)
                .assert_eq(bitwise_and.clone(), local.output[i]);
            builder
                .when(local.is_or)
                .assert_eq(bitwise_or.clone(), local.output[i]);
            builder
                .when(local.is_xor)
                .assert_eq(bitwise_xor.clone(), local.output[i]);
        }

        builder.assert_bool(local.is_and);
        builder.assert_bool(local.is_or);
        builder.assert_bool(local.is_xor);
        builder.assert_bool(local.is_and + local.is_or + local.is_xor);
    }
}
