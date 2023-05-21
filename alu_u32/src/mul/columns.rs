use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct Mul32Cols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// Witnessed output
    pub output: Word<T>,

    /// Witnessed quotients in the congruence relation
    pub r: T,
    pub s: T,

    pub counter: T,
}

pub const NUM_MUL_COLS: usize = size_of::<Mul32Cols<u8>>();
pub const MUL_COL_MAP: Mul32Cols<usize> = make_col_map();

const fn make_col_map() -> Mul32Cols<usize> {
    let indices_arr = indices_arr::<NUM_MUL_COLS>();
    unsafe { transmute::<[usize; NUM_MUL_COLS], Mul32Cols<usize>>(indices_arr) }
}
