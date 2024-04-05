use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

pub struct StaticDataCols<T> {
    // Not used for anything, just present because of the assumption that each chip has a column in its trace
    pub dummy: T,
}

pub const NUM_STATIC_DATA_COLS: usize = size_of::<StaticDataCols<u8>>();
pub const STATIC_DATA_COL_MAP: StaticDataCols<usize> = make_col_map();

const fn make_col_map() -> StaticDataCols<usize> {
    let indices_arr = indices_arr::<NUM_STATIC_DATA_COLS>();
    unsafe { transmute::<[usize; NUM_STATIC_DATA_COLS], StaticDataCols<usize>>(indices_arr) }
}

#[derive(AlignedBorrow, Default)]
pub struct StaticDataPreprocessedCols<T> {
    /// Memory address
    pub addr: T,

    /// Memory cell
    pub value: Word<T>,

    /// Whether this row represents a real (address, value) pair
    pub is_real: T,
}

pub const NUM_STATIC_DATA_PREPROCESSED_COLS: usize = size_of::<StaticDataPreprocessedCols<u8>>();
pub const STATIC_DATA_PREPROCESSED_COL_MAP: StaticDataPreprocessedCols<usize> = make_preprocessed_col_map();

const fn make_preprocessed_col_map() -> StaticDataPreprocessedCols<usize> {
    let indices_arr = indices_arr::<NUM_STATIC_DATA_PREPROCESSED_COLS>();
    unsafe { transmute::<[usize; NUM_STATIC_DATA_PREPROCESSED_COLS], StaticDataPreprocessedCols<usize>>(indices_arr) }
}
