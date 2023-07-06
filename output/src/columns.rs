use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct OutputCols<T> {
    /// Memory address
    pub addr: T,

    /// Memory cell
    pub value: Word<T>,

    pub opcode: T,
}

pub const NUM_OUTPUT_COLS: usize = size_of::<OutputCols<u8>>();
pub const OUTPUT_COL_MAP: OutputCols<usize> = make_col_map();

const fn make_col_map() -> OutputCols<usize> {
    let indices_arr = indices_arr::<NUM_OUTPUT_COLS>();
    unsafe { transmute::<[usize; NUM_OUTPUT_COLS], OutputCols<usize>>(indices_arr) }
}
