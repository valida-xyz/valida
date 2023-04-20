#![no_std]

use crate::columns::CpuCols;
use p3_field::field::Field;

pub mod columns;
mod stark;

pub const OPERAND_ELEMENTS: usize = 5;
pub const INSTRUCTION_ELEMENTS: usize = OPERAND_ELEMENTS + 1;
pub const CPU_MEMORY_CHANNELS: usize = 3;
