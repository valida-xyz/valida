#![no_std]

mod columns;
mod stark;

pub(crate) const INSTRUCTION_ELEMENTS: usize = 6;

pub(crate) const CPU_MEMORY_CHANNELS: usize = 3;
