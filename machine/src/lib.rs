#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;
extern crate self as valida_machine;

pub mod __internal;
mod advice;
mod check_constraints;
mod chip;
mod config;
mod core;
mod debug_builder;
mod folding_builder;
mod machine;
mod program;
mod proof;
mod quotient;
mod symbolic;

pub use advice::*;
pub use chip::*;
pub use config::*;
pub use core::*;
pub use machine::*;
pub use program::*;
pub use proof::*;

pub const OPERAND_ELEMENTS: usize = 5;
pub const INSTRUCTION_ELEMENTS: usize = OPERAND_ELEMENTS + 1;
pub const CPU_MEMORY_CHANNELS: usize = 3;
pub const MEMORY_CELL_BYTES: usize = 4;
pub const LOOKUP_DEGREE_BOUND: usize = 3;
