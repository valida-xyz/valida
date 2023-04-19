use crate::{CPU_MEMORY_CHANNELS, INSTRUCTION_ELEMENTS};
use core::borrow::{Borrow, BorrowMut};
use core::mem::{size_of, transmute};
use valida_machine::Word;
use valida_util::indices_arr;

#[derive(Default)]
pub struct CpuCols<T> {
    /// Program counter.
    pub pc: T,

    /// Frame pointer.
    pub fp: T,

    /// The instruction that was read, i.e. `program_code[pc]`.
    pub instruction: [T; INSTRUCTION_ELEMENTS],

    /// Absolute addresses for memory operations.
    pub addr_read_1: T,
    pub addr_read_2: T,
    pub addr_write: T,

    /// Buffers for the two memory reads and single write.
    pub mem_read_1: Word<T>,
    pub mem_read_2: Word<T>,
    pub mem_write: Word<T>,

    /// Flags indicating what type of operation is being performed this cycle.
    pub opcode_flags: OpcodeFlagCols<T>,

    /// Channels to the memory bus.
    pub mem_channels: [MemoryChannelCols<T>; CPU_MEMORY_CHANNELS],

    /// Channel to the shared chip bus.
    pub chip_channel: ChipChannelCols<T>,
}

impl<T> CpuCols<T> {
    pub fn set_pc(&mut self, pc: T) {
        self.pc = pc;
    }

    /// Set absolute addresses for memory operations.
    pub fn set_addr_read_1(&mut self, addr: T) {
        self.addr_read_1 = addr;
    }
    pub fn set_addr_read_2(&mut self, addr: T) {
        self.addr_read_2 = addr;
    }
    pub fn set_addr_write(&mut self, addr: T) {
        self.addr_write = addr;
    }

    /// Set buffered memory values.
    pub fn set_mem_read_1(&mut self, mem: Word<T>) {
        self.mem_read_1 = mem;
    }
    pub fn set_mem_read_2(&mut self, mem: Word<T>) {
        self.mem_read_2 = mem;
    }
    pub fn set_mem_write(&mut self, mem: Word<T>) {
        self.mem_write = mem;
    }

    pub fn set_opcode_flags(&mut self, operands: &[T]) {
        todo!()
    }
    pub fn set_mem_channel_data(&mut self) {
        todo!()
    }
    pub fn set_chip_channel_data(&mut self) {
        todo!()
    }
}

#[derive(Default)]
pub struct OpcodeFlagCols<T> {
    pub is_imm32: T,
    pub is_bus_op: T,
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

// `u8` is guaranteed to have a `size_of` of 1.
pub const NUM_CPU_COLUMNS: usize = size_of::<CpuCols<u8>>();

pub const CPU_COL_INDICES: CpuCols<usize> = make_col_map();

const fn make_col_map() -> CpuCols<usize> {
    let indices_arr = indices_arr::<NUM_CPU_COLUMNS>();
    unsafe { transmute::<[usize; NUM_CPU_COLUMNS], CpuCols<usize>>(indices_arr) }
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
