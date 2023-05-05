use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(Default)]
pub struct MemoryCols<T> {
    /// Memory address
    pub addr: T,

    /// Memory cell
    pub value: Word<T>,

    /// Main CPU clock cycle
    pub clk: T,

    /// Whether memory operation is a read
    pub is_read: T,

    /// Whether memory operation is a dummy read
    pub is_dummy: T,

    /// Either addr' - addr (if address is changed), or clk' - clk (if address is not changed)
    pub diff: T,
    /// The inverse of `diff`, or 0 if `diff = 0`.
    pub diff_inv: T,

    /// A boolean flag indicating whether addr' - addr == 0
    pub addr_not_equal: T,
}

pub const NUM_MEM_COLS: usize = size_of::<MemoryCols<u8>>();

const fn make_col_map() -> MemoryCols<usize> {
    let indices_arr = indices_arr::<NUM_MEM_COLS>();
    unsafe { transmute::<[usize; NUM_MEM_COLS], MemoryCols<usize>>(indices_arr) }
}

impl<T> Borrow<MemoryCols<T>> for [T] {
    fn borrow(&self) -> &MemoryCols<T> {
        // TODO: Double check if this is correct & consider making asserts debug-only.
        let (prefix, shorts, _suffix) = unsafe { self.align_to::<MemoryCols<T>>() };
        assert!(prefix.is_empty(), "Data was not aligned");
        assert_eq!(shorts.len(), 1);
        &shorts[0]
    }
}

impl<T> BorrowMut<MemoryCols<T>> for [T] {
    fn borrow_mut(&mut self) -> &mut MemoryCols<T> {
        // TODO: Double check if this is correct & consider making asserts debug-only.
        let (prefix, shorts, _suffix) = unsafe { self.align_to_mut::<MemoryCols<T>>() };
        assert!(prefix.is_empty(), "Data was not aligned");
        assert_eq!(shorts.len(), 1);
        &mut shorts[0]
    }
}
