use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct Sub32Cols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// Witnessed output
    pub output: Word<T>,

    pub is_real: T,
}

pub const NUM_SUB_COLS: usize = size_of::<Sub32Cols<u8>>();
pub const SUB_COL_MAP: Sub32Cols<usize> = make_col_map();

const fn make_col_map() -> Sub32Cols<usize> {
    let indices_arr = indices_arr::<NUM_SUB_COLS>();
    unsafe { transmute::<[usize; NUM_SUB_COLS], Sub32Cols<usize>>(indices_arr) }
}
