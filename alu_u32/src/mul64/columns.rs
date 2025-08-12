extern crate alloc;

use alloc::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(Default)]
pub struct Word64<T> {
    pub most_significant: Word<T>,
    pub least_significant: Word<T>,
}

#[derive(AlignedBorrow, Default)]
pub struct Mul64Cols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,
    pub output: Word64<T>,
    // TODO
}

pub const NUM_MUL64_COLS: usize = size_of::<Mul64Cols<u8>>();
pub const MUL64_COL_MAP: Mul64Cols<usize> = make_col_map();

const fn make_col_map() -> Mul64Cols<usize> {
    let indices_arr = indices_arr::<NUM_MUL64_COLS>();
    unsafe { transmute::<[usize; NUM_MUL64_COLS], Mul64Cols<usize>>(indices_arr) }
}
