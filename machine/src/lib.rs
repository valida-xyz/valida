#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;
extern crate self as valida_machine;

pub use crate::core::Word;
pub use chip::{BusArgument, Chip, Interaction, InteractionType, ValidaAirBuilder};
pub use p3_field::{
    AbstractExtensionField, AbstractField, ExtensionField, Field, PrimeField, PrimeField64,
};
// TODO: some are also re-exported, so they shouldn't be pub?
pub mod __internal;
mod advice;
mod check_constraints;
mod chip;
pub mod config;
pub mod core;
mod debug_builder;
mod folding_builder;
mod machine;
mod program;
pub mod proof;
mod quotient;
mod symbolic;

pub use advice::*;
pub use chip::*;
pub use core::*;
pub use machine::*;
pub use program::*;

pub const OPERAND_ELEMENTS: usize = 5;
pub const INSTRUCTION_ELEMENTS: usize = OPERAND_ELEMENTS + 1;
pub const CPU_MEMORY_CHANNELS: usize = 3;
pub const MEMORY_CELL_BYTES: usize = 4;
pub const LOOKUP_DEGREE_BOUND: usize = 3;
