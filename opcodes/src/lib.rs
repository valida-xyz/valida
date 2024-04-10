use num_enum::TryFromPrimitive;

pub const BYTES_PER_INSTR: u32 = 24; // 4 bytes per word * 6 words per instruction

#[repr(u32)]
#[derive(Debug, TryFromPrimitive)]
pub enum Opcode {
    LOAD32 = 1,
    STORE32 = 2,
    JAL = 3,
    JALV = 4,
    BEQ = 5,
    BNE = 6,
    IMM32 = 7,
    STOP = 8,
    READ_ADVICE = 9,
    LOADFP = 10,
    LOADU8 = 11,
    LOADS8 = 12,
    STOREU8 = 13,

    ADD32 = 100,
    SUB32 = 101,
    MUL32 = 102,
    DIV32 = 103,
    SDIV32 = 110,
    LT32 = 104,
    SHL32 = 105,
    SHR32 = 106,
    AND32 = 107,
    OR32 = 108,
    XOR32 = 109,
    NE32 = 111,
    MULHU32 = 112,
    SRA32 = 113,
    MULHS32 = 114,
    LTE32 = 115,
    EQ32 = 116,
    ADD = 200,
    SUB = 201,
    MUL = 202,
    WRITE = 300,
}

macro_rules! declare_opcode {
    ($opcode : ident) => {
        pub const $opcode: u32 = Opcode::$opcode as u32;
    };
}

// TODO: should combine enum together

/// CORE
declare_opcode!(LOAD32);
declare_opcode!(STORE32);
declare_opcode!(JAL);
declare_opcode!(JALV);
declare_opcode!(BEQ);
declare_opcode!(BNE);
declare_opcode!(IMM32);
declare_opcode!(STOP);
declare_opcode!(LOADFP);
declare_opcode!(LOADU8);
declare_opcode!(LOADS8);
declare_opcode!(STOREU8);

/// NONDETERMINISTIC
declare_opcode!(READ_ADVICE);

/// U32 ALU
declare_opcode!(ADD32);
declare_opcode!(SUB32);
declare_opcode!(MUL32);
declare_opcode!(DIV32);
declare_opcode!(SDIV32);
declare_opcode!(LT32);
declare_opcode!(SHL32);
declare_opcode!(SHR32);
declare_opcode!(AND32);
declare_opcode!(OR32);
declare_opcode!(XOR32);
declare_opcode!(NE32);
declare_opcode!(MULHU32);
declare_opcode!(SRA32);
declare_opcode!(MULHS32);
declare_opcode!(LTE32);
declare_opcode!(EQ32);

/// NATIVE FIELD
declare_opcode!(ADD);
declare_opcode!(SUB);
declare_opcode!(MUL);

/// OUTPUT
declare_opcode!(WRITE);
