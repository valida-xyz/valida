use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct Div32Cols<T> {
    //input_1 = input_2*output + q
    //intermediate_output = input_2*output
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// Witnessed output
    pub output: Word<T>,
    pub intermediate_output: Word<T>,
    pub q: Word<T>,
    pub is_div: T,
    pub is_sdiv: T,
}

pub const NUM_DIV_COLS: usize = size_of::<Div32Cols<u8>>();
pub const DIV_COL_MAP: Div32Cols<usize> = make_col_map();

const fn make_col_map() -> Div32Cols<usize> {
    let indices_arr = indices_arr::<NUM_DIV_COLS>();
    unsafe { transmute::<[usize; NUM_DIV_COLS], Div32Cols<usize>>(indices_arr) }
}
