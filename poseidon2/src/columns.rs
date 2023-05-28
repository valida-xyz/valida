//! Posiedon2 STARK Columns

use valida_derive::AlignedBorrow;
use valida_util::indices_arr;

/// Columns
#[repr(C)]
#[derive(AlignedBorrow, Default)]
pub struct Columns<T> {
    
}

/// Number of Columns
pub const NUM_COLUMNS = size_of::<Columns<u8>>(); 

/// Column Indices
pub const COLUMN_INDICES: Columns<usize> = make_column_map();

/// Builds the column map from the index array.
#[inline]
const fn make_column_map() -> Columns<usize> {
    let indices = indices_arr::<NUM_COLUMNS>();
    unsafe { transmute::<[usize; NUM_COLUMNS], Columns<usize>>(indices) }
}
