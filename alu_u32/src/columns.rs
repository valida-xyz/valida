use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct AluU32Cols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// Witnessed output
    pub output: [T; 8],

    /// Witnessed quotient in the congruence relation
    pub s: T,
}

pub const NUM_ALU_COLS: usize = size_of::<AluU32Cols<u8>>();
pub const ALU_COL_MAP: AluU32Cols<usize> = make_col_map();

const fn make_col_map() -> AluU32Cols<usize> {
    let indices_arr = indices_arr::<NUM_ALU_COLS>();
    unsafe { transmute::<[usize; NUM_ALU_COLS], AluU32Cols<usize>>(indices_arr) }
}
