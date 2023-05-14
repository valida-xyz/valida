#![no_std]

extern crate alloc;

use valida_alu_u32::{ALU32Chip, MachineWithALU32Chip};
use valida_alu_u32::{Add32Instruction, Mul32Instruction};
use valida_bus::{CpuMemBus, SharedCoprocessorBus};
use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    Load32Instruction, Store32Instruction,
};
use valida_cpu::{CpuChip, MachineWithCpuChip};
use valida_derive::Machine;
use valida_machine::{Field32, Instruction, Machine, Operands, ProgramROM, ProgramState};
use valida_memory::{MachineWithMemoryChip, MemoryChip};

// TODO: Emit instruction members in the derive macro instead of manually including

#[derive(Machine)]
pub struct BasicMachine {
    // Core instructions
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

    // ALU instructions
    #[instruction(alu_u32)]
    add32: Add32Instruction,
    #[instruction(alu_u32)]
    mul32: Mul32Instruction,

    #[chip]
    cpu: CpuChip,
    #[chip]
    mem: MemoryChip,
    #[chip]
    alu_u32: ALU32Chip,

    #[bus(cpu, mem)]
    cpu_mem_bus: CpuMemBus,
    #[bus(cpu, alu_u32)]
    cpu_alu_u32_bus: SharedCoprocessorBus,
}
