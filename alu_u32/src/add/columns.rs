use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct Add32Cols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    pub carry: [T; 3],

    /// Witnessed output
    pub output: Word<T>,

    pub opcode: T,
}

pub const NUM_ADD_COLS: usize = size_of::<Add32Cols<u8>>();
pub const ADD_COL_MAP: Add32Cols<usize> = make_col_map();

const fn make_col_map() -> Add32Cols<usize> {
    let indices_arr = indices_arr::<NUM_ADD_COLS>();
    unsafe { transmute::<[usize; NUM_ADD_COLS], Add32Cols<usize>>(indices_arr) }
}
