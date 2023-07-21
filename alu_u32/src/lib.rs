#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use p3_field::AbstractField;

const ADD32_OPCODE: u32 = 9;
const SUB32_OPCODE: u32 = 10;
const MUL32_OPCODE: u32 = 11;
#[allow(dead_code)]
const LT_OPCODE: u32 = 12;

pub mod add;
pub mod lt;
pub mod mul;
pub mod sub;

fn pad_to_power_of_two<const N: usize, F: AbstractField>(values: &mut Vec<F>) {
    let n_real_rows = values.len() / N;
    values.resize(n_real_rows.next_power_of_two() * N, F::ZERO);
}
