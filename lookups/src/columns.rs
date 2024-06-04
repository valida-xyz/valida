use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default, Debug)]
pub struct LookupCols<T> {
    // the multiplicity of the entry
    pub mult: T,
}

pub const NUM_LOOKUP_COLS: usize = size_of::<LookupCols<u8>>();
pub const LOOKUP_COL_MAP: LookupCols<usize> = make_col_map();

const fn make_col_map() -> LookupCols<usize> {
    let indices_arr = indices_arr::<NUM_LOOKUP_COLS>();
    unsafe { transmute::<[usize; NUM_LOOKUP_COLS], LookupCols<usize>>(indices_arr) }
}
