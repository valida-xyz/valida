use core::borrow::{Borrow, BorrowMut};
use core::iter;
use core::mem::{size_of, transmute};
use valida_derive::AlignedBorrow;
use valida_machine::{Operands, Word, CPU_MEMORY_CHANNELS};
use valida_util::indices_arr;

#[repr(C)]
#[derive(AlignedBorrow, Default)]
pub struct CpuCols<T> {
    /// Clock cycle
    pub clk: T,

    /// Program counter.
    pub pc: T,

    /// Frame pointer.
    pub fp: T,

    /// An immediate value
    pub imm: Word<T>,

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
    pub is_imm_op: T,
    pub is_load: T,
    pub is_store: T,
    pub is_beq: T,
    pub is_bne: T,
    pub is_jal: T,
    pub is_jalv: T,
}

#[derive(Default)]
pub struct MemoryChannelCols<T> {
    pub used: T,
    pub is_read: T,
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

impl<T: Copy> ChipChannelCols<T> {
    pub(crate) fn iter_flat(&self) -> impl Iterator<Item = T> {
        iter::once(self.opcode)
            .chain(self.read_value_1.0.into_iter())
            .chain(self.read_value_2.0.into_iter())
            .chain(self.write_value.0.into_iter())
    }
}

impl<T: Copy> CpuCols<T> {
    pub fn read_addr_1(&self) -> T {
        self.mem_channels[0].addr
    }
    pub fn read_addr_2(&self) -> T {
        self.mem_channels[1].addr
    }
    pub fn write_addr(&self) -> T {
        self.mem_channels[2].addr
    }

    pub fn read_value_1(&self) -> Word<T> {
        self.mem_channels[0].value
    }
    pub fn read_value_2(&self) -> Word<T> {
        self.mem_channels[1].value
    }
    pub fn write_value(&self) -> Word<T> {
        self.mem_channels[2].value
    }

    pub fn read_1_used(&self) -> T {
        self.mem_channels[0].used
    }
    pub fn read_2_used(&self) -> T {
        self.mem_channels[1].used
    }
    pub fn write_used(&self) -> T {
        self.mem_channels[2].used
    }
}

// `u8` is guaranteed to have a `size_of` of 1.
pub const NUM_CPU_COLS: usize = size_of::<CpuCols<u8>>();
pub const NUM_CPU_PERM_COLS: usize = 0; // todo!();

pub const CPU_COL_INDICES: CpuCols<usize> = make_col_map();

const fn make_col_map() -> CpuCols<usize> {
    let indices_arr = indices_arr::<NUM_CPU_COLS>();
    unsafe { transmute::<[usize; NUM_CPU_COLS], CpuCols<usize>>(indices_arr) }
}
