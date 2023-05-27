#![no_std]

const Add32Opcode: u32 = 8;
const Sub32Opcode: u32 = 9;
const Mul32Opcode: u32 = 10;
const LtOpcode: u32 = 11;

pub mod add;
pub mod lt;
pub mod mul;
pub mod sub;
