#[derive(AlignedBorrow, Default)]
pub struct StaticDataCols<T> {
    /// Memory address
    pub addr: T,

    /// Memory cell
    pub value: Word<T>,
}

pub const NUM_STATIC_DATA_COLS: usize = size_of::<StaticDataCols<u8>>();
pub const STATIC_DATA_COL_MAP: StaticDataCols<usize> = make_col_map();

const fn make_col_map() -> StaticDataCols<usize> {
    let indices_arr = indices_arr::<NUM_STATIC_DATA_COLS>();
    unsafe { transmute::<[usize; NUM_STATIC_DATA_COLS], MemoryCols<usize>>(indices_arr) }
}
