#![no_std]

const ADD32_OPCODE: u32 = 9;
const SUB32_OPCODE: u32 = 10;
const MUL32_OPCODE: u32 = 11;
#[allow(dead_code)]
const LT_OPCODE: u32 = 12;

pub mod add;
pub mod lt;
pub mod mul;
pub mod sub;
