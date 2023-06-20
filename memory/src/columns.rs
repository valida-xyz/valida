use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(AlignedBorrow, Default)]
pub struct MemoryCols<T> {
    /// Memory address
    pub addr: T,

    /// Memory cell
    pub value: Word<T>,

    /// Main CPU clock cycle
    pub clk: T,

    /// Whether memory operation is a read
    pub is_read: T,

    /// Whether memory operation is a real read, not a dummy.
    pub is_real: T,

    /// Either addr' - addr (if address is changed), or clk' - clk (if address is not changed)
    pub diff: T,
    /// The inverse of `diff`, or 0 if `diff = 0`.
    pub diff_inv: T,

    /// A boolean flag indicating whether addr' - addr == 0
    pub addr_not_equal: T,

    /// Counter and lookup multiplicities for local range check
    pub counter: T,
    pub counter_mult: T,
}

pub const NUM_MEM_COLS: usize = size_of::<MemoryCols<u8>>();
pub const MEM_COL_MAP: MemoryCols<usize> = make_col_map();

const fn make_col_map() -> MemoryCols<usize> {
    let indices_arr = indices_arr::<NUM_MEM_COLS>();
    unsafe { transmute::<[usize; NUM_MEM_COLS], MemoryCols<usize>>(indices_arr) }
}
