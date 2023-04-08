use core::mem::{size_of, transmute};
use crate::cpu::CPU_MEMORY_CHANNELS;
use crate::memory::MEMORY_CELL_BYTES;
use crate::util::indices_arr;

pub struct CpuCols<T> {
    pub instruction_pointer: T,
    pub opcode_flags: OpcodeFlagCols<T>,
    pub mem_channels: [MemoryChannelCols<T>; CPU_MEMORY_CHANNELS],
}

pub struct OpcodeFlagCols<T> {
    pub is_bus_op: T,
}

pub struct MemoryChannelCols<T> {
    pub used: T,
    pub addr: T,
    pub value: [T; MEMORY_CELL_BYTES],
}

// `u8` is guaranteed to have a `size_of` of 1.
pub const NUM_CPU_COLUMNS: usize = size_of::<CpuCols<u8>>();

pub const CPU_COL_INDICES: CpuCols<usize> = make_col_map();

const fn make_col_map() -> CpuCols<usize> {
    let indices_arr = indices_arr::<NUM_CPU_COLUMNS>();
    unsafe { transmute::<[usize; NUM_CPU_COLUMNS], CpuCols<usize>>(indices_arr) }
}
