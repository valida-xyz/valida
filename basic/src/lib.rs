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
use valida_machine::{Instruction, Machine, Operands, ProgramROM};
use valida_memory::{MachineWithMemoryChip, MemoryChip};

// TODO: Emit instruction members in the derive macro instead of manually including

#[derive(Machine, Default)]
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

#[cfg(test)]
mod tests {
    #[test]
    fn store32() {
        use super::*;
        use alloc::vec;
        use p3_field::AbstractField;
        use p3_mersenne_31::Mersenne31 as Fp;
        use valida_machine::InstructionWord;

        let mut machine = BasicMachine::default();

        let program = vec![
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-4, 0, 0, 0, 42]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::from_i32_slice(&[0, -8, -4, 0, 0]),
            },
            InstructionWord {
                opcode: 0,
                operands: Operands::default(),
            },
        ];
        let rom = ProgramROM::new(program);

        machine.run(rom);
    }
}
