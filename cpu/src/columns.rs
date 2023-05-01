use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_machine::{Operands, Word, CPU_MEMORY_CHANNELS};
use valida_util::indices_arr;

#[derive(Default)]
pub struct CpuCols<T> {
    /// Program counter.
    pub pc: T,

    /// Frame pointer.
    pub fp: T,

    /// The instruction that was read, i.e. `program_code[pc]`.
    pub instruction: InstructionCols<T>,

    /// Flags indicating what type of operation is being performed this cycle.
    pub opcode_flags: OpcodeFlagCols<T>,

    /// When doing an equality test between two words, `x` and `y`, this holds the sum of
    /// `(x_i - y_i)^2`, which is zero if and only if `x = y`.
    pub diff: T,
    /// The inverse of `diff`, or undefined if `diff = 0`.
    pub diff_inv: T,
    /// A boolean flag indicating whether `diff != 0`.
    pub not_equal: T,

    /// Channels to the memory bus.
    pub mem_channels: [MemoryChannelCols<T>; CPU_MEMORY_CHANNELS],

    /// Channel to the shared chip bus.
    pub chip_channel: ChipChannelCols<T>,
}

#[derive(Default)]
pub struct InstructionCols<F> {
    pub opcode: F,
    pub operands: Operands<F>,
}

#[derive(Default)]
pub struct OpcodeFlagCols<T> {
    pub is_imm32: T,
    pub is_bus_op: T,
    pub is_beq: T,
    pub is_bne: T,
    pub is_jal: T,
    pub is_jalv: T,
}

#[derive(Default)]
pub struct MemoryChannelCols<T> {
    pub used: T,
    pub addr: T,
    pub value: Word<T>,
}

#[derive(Default)]
pub struct ChipChannelCols<T> {
    pub opcode: T,
    pub read_value_1: Word<T>,
    pub read_value_2: Word<T>,
    pub write_value: Word<T>,
}

impl<T: Copy> CpuCols<T> {
    pub fn mem_read_1(&self) -> Word<T> {
        self.mem_channels[0].value
    }
    pub fn mem_read_2(&self) -> Word<T> {
        self.mem_channels[1].value
    }
    pub fn mem_write(&self) -> Word<T> {
        self.mem_channels[2].value
    }
}

// `u8` is guaranteed to have a `size_of` of 1.
pub const NUM_CPU_COLS: usize = size_of::<CpuCols<u8>>();

pub const CPU_COL_INDICES: CpuCols<usize> = make_col_map();

const fn make_col_map() -> CpuCols<usize> {
    let indices_arr = indices_arr::<NUM_CPU_COLS>();
    unsafe { transmute::<[usize; NUM_CPU_COLS], CpuCols<usize>>(indices_arr) }
}

impl<T> Borrow<CpuCols<T>> for [T] {
    fn borrow(&self) -> &CpuCols<T> {
        // TODO: Double check if this is correct & consider making asserts debug-only.
        let (prefix, shorts, _suffix) = unsafe { self.align_to::<CpuCols<T>>() };
        assert!(prefix.is_empty(), "Data was not aligned");
        assert_eq!(shorts.len(), 1);
        &shorts[0]
    }
}

impl<T> BorrowMut<CpuCols<T>> for [T] {
    fn borrow_mut(&mut self) -> &mut CpuCols<T> {
        // TODO: Double check if this is correct & consider making asserts debug-only.
        let (prefix, shorts, _suffix) = unsafe { self.align_to_mut::<CpuCols<T>>() };
        assert!(prefix.is_empty(), "Data was not aligned");
        assert_eq!(shorts.len(), 1);
        &mut shorts[0]
    }
}
