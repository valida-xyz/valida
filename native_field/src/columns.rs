use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct NativeFieldCols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// Witnessed output
    pub output: Word<T>,

    pub is_add: T,
    pub is_sub: T,
    pub is_mul: T,
}

pub const NUM_COLS: usize = size_of::<NativeFieldCols<u8>>();
pub const COL_MAP: NativeFieldCols<usize> = make_col_map();

const fn make_col_map() -> NativeFieldCols<usize> {
    let indices_arr = indices_arr::<NUM_COLS>();
    unsafe { transmute::<[usize; NUM_COLS], NativeFieldCols<usize>>(indices_arr) }
}
