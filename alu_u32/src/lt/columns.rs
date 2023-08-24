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
    pub byte_flag: [T; 3],

    /// Bit decomposition of 256 + input_1 - input_2
    pub bits: [T; 10],

    pub output: T,

    pub multiplicity: T,
}

pub const NUM_LT_COLS: usize = size_of::<Lt32Cols<u8>>();
pub const LT_COL_MAP: Lt32Cols<usize> = make_col_map();

const fn make_col_map() -> Lt32Cols<usize> {
    let indices_arr = indices_arr::<NUM_LT_COLS>();
    unsafe { transmute::<[usize; NUM_LT_COLS], Lt32Cols<usize>>(indices_arr) }
}
