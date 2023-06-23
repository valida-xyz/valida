#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use p3_field::Field;

/// Returns `[0, ..., N - 1]`.
pub const fn indices_arr<const N: usize>() -> [usize; N] {
    let mut indices_arr = [0; N];
    let mut i = 0;
    while i < N {
        indices_arr[i] = i;
        i += 1;
    }
    indices_arr
}

pub fn batch_multiplicative_inverse<F: Field>(values: Vec<F>) -> Vec<F> {
    // TODO: Handle zero values correctly
    p3_field::batch_multiplicative_inverse(&values)
}
