use core::mem::{size_of, transmute};
use valida_util::indices_arr;

#[derive(Default)]
pub struct RangeCols<T> {
    pub mult: T, // Multiplicity
    pub counter: T,
}

pub struct RangePreprocessedCols {
    // TODO
}

pub const NUM_RANGE_COLS: usize = size_of::<RangeCols<u8>>();
pub const RANGE_COL_MAP: RangeCols<usize> = make_col_map();

const fn make_col_map() -> RangeCols<usize> {
    let indices_arr = indices_arr::<NUM_RANGE_COLS>();
    unsafe { transmute::<[usize; NUM_RANGE_COLS], RangeCols<usize>>(indices_arr) }
}
