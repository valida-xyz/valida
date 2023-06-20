#![no_std]

extern crate alloc;

use alloc::vec;
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

// TODO: Handle zero values correctly
pub fn batch_invert<F: Field>(values: Vec<F>) -> Vec<F> {
    let mut res = vec![F::ZERO; values.len()];
    let mut prod = F::ONE;
    for (n, value) in values.iter().cloned().enumerate() {
        res[n] = prod;
        prod *= value;
    }
    let mut inv = prod.inverse();
    for (n, value) in values.iter().cloned().rev().enumerate().rev() {
        res[n] *= inv;
        inv *= value;
    }
    res
}
