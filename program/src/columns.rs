use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Operands;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct ProgramCols<T> {
    pub multiplicity: T,
}

#[derive(AlignedBorrow, Default)]
pub struct ProgramPreprocessedCols<T> {
    pub pc: T,
    pub opcode: T,
    pub operands: Operands<T>,
}

pub const NUM_COLS: usize = size_of::<ProgramCols<u8>>();
pub const COL_MAP: ProgramCols<usize> = make_col_map();

pub const NUM_PREPROCESSED_COLS: usize = size_of::<ProgramPreprocessedCols<u8>>();
pub const PREPROCESSED_COL_MAP: ProgramPreprocessedCols<usize> = make_preprocessed_col_map();

const fn make_col_map() -> ProgramCols<usize> {
    let indices_arr = indices_arr::<NUM_COLS>();
    unsafe { transmute::<[usize; NUM_COLS], ProgramCols<usize>>(indices_arr) }
}

const fn make_preprocessed_col_map() -> ProgramPreprocessedCols<usize> {
    let indices_arr = indices_arr::<NUM_PREPROCESSED_COLS>();
    unsafe {
        transmute::<[usize; NUM_PREPROCESSED_COLS], ProgramPreprocessedCols<usize>>(indices_arr)
    }
}
