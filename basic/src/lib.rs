#![no_std]

extern crate alloc;

use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    Load32Instruction, Store32Instruction,
};
use valida_cpu::{CpuChip, MachineWithCpuChip};
use valida_derive::Machine;
use valida_machine::{Instruction, Machine, Operands};
use valida_memory::{MachineWithMemoryChip, MemoryChip};

// TODO: Emit instruction members in the derive macro instead of manually including

#[derive(Machine)]
pub struct BasicMachine {
    #[instruction]
    load32: Load32Instruction,
    #[instruction]
    store32: Store32Instruction,
    #[instruction]
    jal: JalInstruction,
    #[instruction]
    jalv: JalvInstruction,
    #[instruction]
    beq: BeqInstruction,
    #[instruction]
    bne: BneInstruction,
    #[instruction]
    imm32: Imm32Instruction,

    #[chip]
    cpu: CpuChip,
    #[chip]
    mem: MemoryChip,
}
