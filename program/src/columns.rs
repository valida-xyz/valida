use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Operands;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct ProgramCols<T> {
    pub pc: T,
    pub opcode: T,
    pub operands: Operands<T>,
    pub multiplicity: T,
}

pub const NUM_PROGRAM_COLS: usize = size_of::<ProgramCols<u8>>();
pub const COL_MAP: ProgramCols<usize> = make_col_map();

const fn make_col_map() -> ProgramCols<usize> {
    let indices_arr = indices_arr::<NUM_PROGRAM_COLS>();
    unsafe { transmute::<[usize; NUM_PROGRAM_COLS], ProgramCols<usize>>(indices_arr) }
}
