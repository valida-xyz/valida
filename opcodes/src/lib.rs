/// CORE
pub const LOAD32: u32 = 1;
pub const STORE32: u32 = 2;
pub const JAL: u32 = 3;
pub const JALV: u32 = 4;
pub const BEQ: u32 = 5;
pub const BNE: u32 = 6;
pub const IMM32: u32 = 7;
pub const STOP: u32 = 8;

/// NONDETERMINISTIC
pub const READ_ADVICE: u32 = 9;

/// U32 ALU
pub const ADD32: u32 = 100;
pub const SUB32: u32 = 101;
pub const MUL32: u32 = 102;
pub const DIV32: u32 = 103;
pub const SDIV32: u32 = 110;
pub const LT32: u32 = 104;
pub const SHL32: u32 = 105;
pub const SHR32: u32 = 106;
pub const AND32: u32 = 107;
pub const OR32: u32 = 108;
pub const XOR32: u32 = 109;
pub const MULHU32 : u32 = 112;
pub const SRA32 : u32 = 113;
pub const MULHS32 : u32 =114;

/// NATIVE FIELD
pub const ADD: u32 = 200;
pub const SUB: u32 = 201;
pub const MUL: u32 = 202;

/// OUTPUT
pub const WRITE: u32 = 300;
