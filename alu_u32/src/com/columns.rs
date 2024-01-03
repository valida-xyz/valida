use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct Com32Cols<T> {
    pub input_1: Word<T>,
    pub input_2: Word<T>,

    /// Boolean flags indicating which byte pair differs
    pub byte_flag: [T; 3],

    /// Bit decomposition of 256 + input_1 - input_2
    pub bits: [T; 10],

    pub output: T,

    pub multiplicity: T,
}

pub const NUM_COM_COLS: usize = size_of::<Com32Cols<u8>>();
pub const COM_COL_MAP: Com32Cols<usize> = make_col_map();

const fn make_col_map() -> Com32Cols<usize> {
    let indices_arr = indices_arr::<NUM_COM_COLS>();
    unsafe { transmute::<[usize; NUM_COM_COLS], Com32Cols<usize>>(indices_arr) }
}
