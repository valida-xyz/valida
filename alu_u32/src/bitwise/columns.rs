use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct Bitwise32Cols<T> {
    // Inputs
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// Aggregated Output
    pub output: Word<T>,
    pub is_and: T,
    pub is_or: T,
    pub is_xor: T,

    // Lookups
    pub byte_lookup: T,
    pub byte_mult: T,

    pub and: Word<T>,
    pub and_lookup: T,
    pub and_mult: T,

    pub or: Word<T>,
    pub or_lookup: T,
    pub or_mult: T,

    pub xor: Word<T>,
    pub xor_lookup: T,
    pub xor_mult: T,
}

pub const NUM_COLS: usize = size_of::<Bitwise32Cols<u8>>();
pub const COL_MAP: Bitwise32Cols<usize> = make_col_map();

const fn make_col_map() -> Bitwise32Cols<usize> {
    let indices_arr = indices_arr::<NUM_COLS>();
    unsafe { transmute::<[usize; NUM_COLS], Bitwise32Cols<usize>>(indices_arr) }
}
