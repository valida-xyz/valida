#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use p3_field::AbstractField;

mod mersenne_31;

fn pad_to_power_of_two<const N: usize, F: AbstractField>(values: &mut Vec<F>) {
    let n_real_rows = values.len() / N;
    values.resize(n_real_rows.next_power_of_two() * N, F::ZERO);
}
