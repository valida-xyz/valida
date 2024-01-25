use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct Com32Cols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// When doing an equality test between two words, `x` and `y`, this holds the sum of
    /// `(x_i - y_i)^2`, which is zero if and only if `x = y`.
    pub diff: T,
    /// The inverse of `diff`, or undefined if `diff = 0`.
    pub diff_inv: T,
    /// A boolean flag indicating whether `diff != 0`.
    pub not_equal: T,

    pub output: T,

    pub is_ne: T,
    pub is_eq: T,
}

pub const NUM_COM_COLS: usize = size_of::<Com32Cols<u8>>();
pub const COM_COL_MAP: Com32Cols<usize> = make_col_map();

const fn make_col_map() -> Com32Cols<usize> {
    let indices_arr = indices_arr::<NUM_COM_COLS>();
    unsafe { transmute::<[usize; NUM_COM_COLS], Com32Cols<usize>>(indices_arr) }
}
