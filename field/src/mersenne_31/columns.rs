use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct Mersenne31Cols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// Witnessed output
    pub output: Word<T>,

    pub opcode: T,
}

pub const NUM_COLS: usize = size_of::<Mersenne31Cols<u8>>();
pub const COL_MAP: Mersenne31Cols<usize> = make_col_map();

const fn make_col_map() -> Mersenne31Cols<usize> {
    let indices_arr = indices_arr::<NUM_COLS>();
    unsafe { transmute::<[usize; NUM_COLS], Mersenne31Cols<usize>>(indices_arr) }
}
