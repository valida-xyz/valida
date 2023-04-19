#![no_std]

use crate::columns::CpuCols;
use p3_field::field::Field;

pub mod columns;
mod stark;

pub const INSTRUCTION_ELEMENTS: usize = 6;
pub const CPU_MEMORY_CHANNELS: usize = 3;

pub struct Cpu {}

impl Cpu {
    pub fn load32<F: Field>(mut row: CpuCols<F>) {
        todo!()
    }

    pub fn store32() {}

    pub fn jal() {}

    pub fn jalv() {}

    pub fn beq() {}

    pub fn bne() {}

    pub fn imm32() {}
}
