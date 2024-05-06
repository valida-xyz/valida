use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct Lt32Cols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// Boolean flags indicating which byte pair differs
    pub byte_flag: [T; 4],

    /// Bit decomposition of 256 + input_1 - input_2
    pub bits: [T; 9],

    pub output: T,

    pub multiplicity: T,

    pub is_lt: T,
    pub is_lte: T,
    pub is_slt: T,
    pub is_sle: T,

    // inverse of input_1[i] - input_2[i] where i is the first byte that differs
    pub diff_inv: T,

    // bit decomposition of top bytes for input_1 and input_2
    pub top_bits_1: [T; 8],
    pub top_bits_2: [T; 8],

    // boolean flag for whether the sign of the two inputs is different
    pub different_signs: T,
}

pub const NUM_LT_COLS: usize = size_of::<Lt32Cols<u8>>();
pub const LT_COL_MAP: Lt32Cols<usize> = make_col_map();

const fn make_col_map() -> Lt32Cols<usize> {
    let indices_arr = indices_arr::<NUM_LT_COLS>();
    unsafe { transmute::<[usize; NUM_LT_COLS], Lt32Cols<usize>>(indices_arr) }
}
