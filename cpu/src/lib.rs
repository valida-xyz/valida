#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use p3_field::field::Field;
use p3_mersenne_31::Mersenne31 as Fp;
use valida_machine::{Instruction, Operands, Word};
use valida_memory::MachineWithMemoryChip;

pub mod columns;
mod stark;

pub enum Operation {
    Store32,
    Load32,
    Jal,
    Jalv,
    Beq,
    Bne,
    Imm32,
}

pub struct CpuChip {
    pub clock: Fp,
    pub pc: Fp,
    pub fp: Fp,
    pub operations: Vec<Operation>,
}

pub trait MachineWithCpuChip: MachineWithMemoryChip {
    fn cpu(&self) -> &CpuChip;
    fn cpu_mut(&mut self) -> &mut CpuChip;
}

pub struct Load32Instruction;
pub struct Store32Instruction;
pub struct JalInstruction;
pub struct JalvInstruction;
pub struct BeqInstruction;
pub struct BneInstruction;
pub struct Imm32Instruction;

impl<M: MachineWithCpuChip> Instruction<M> for Load32Instruction {
    const OPCODE: u32 = 1;

    fn execute(state: &mut M, ops: Operands) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.c();
        let read_addr_2 = state.mem_mut().read(clk, read_addr_1, true);
        let write_addr = state.cpu().fp + ops.a();
        let cell = state.mem_mut().read(clk, read_addr_2, true);
        state.mem_mut().write(clk, write_addr, cell, true);
        state.cpu_mut().pc += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Load32);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for Store32Instruction {
    const OPCODE: u32 = 2;

    fn execute(state: &mut M, ops: Operands) {
        let clk = state.cpu().clock;
        let read_addr = state.cpu().fp + ops.c();
        let write_addr = state.cpu().fp + ops.b();
        let cell = state.mem_mut().read(clk, read_addr, true);
        state.mem_mut().write(clk, write_addr, cell, true);
        state.cpu_mut().pc += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Store32);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for JalInstruction {
    const OPCODE: u32 = 3;

    fn execute(state: &mut M, ops: Operands) {
        let clk = state.cpu().clock;
        // Store pc + 1 to local stack variable at offset a
        let write_addr = state.cpu().fp + ops.a();
        let next_pc = state.cpu().pc + Fp::ONE;
        state.mem_mut().write(clk, write_addr, next_pc, true);
        // Set pc to the field element b
        state.cpu_mut().pc = ops.b();
        // Set fp to fp + c
        state.cpu_mut().fp += ops.c();
        state.cpu_mut().operations.push(Operation::Jal);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for JalvInstruction {
    const OPCODE: u32 = 4;

    fn execute(state: &mut M, ops: Operands) {
        let clk = state.cpu().clock;
        // Store pc + 1 to local stack variable at offset a
        let write_addr = state.cpu().fp + ops.a();
        let next_pc = state.cpu().pc + Fp::ONE;
        state.mem_mut().write(clk, write_addr, next_pc, true);
        // Set pc to the field element [b]
        let read_addr = state.cpu().fp + ops.b();
        state.cpu_mut().pc = state.mem_mut().read(clk, read_addr, true).into();
        // Set fp to [c]
        let read_addr = state.cpu().fp + ops.c();
        state.cpu_mut().fp = state.mem_mut().read(clk, read_addr, true).into();
        state.cpu_mut().operations.push(Operation::Jalv);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for BeqInstruction {
    const OPCODE: u32 = 5;

    fn execute(state: &mut M, ops: Operands) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.b();
        let cell_1 = state.mem_mut().read(clk, read_addr_1, true);
        let cell_2 = if ops.is_imm() == Fp::ONE {
            ops.c().into()
        } else {
            let read_addr_2 = state.cpu().fp + ops.c();
            state.mem_mut().read(clk, read_addr_2, true)
        };
        if cell_1 == cell_2 {
            state.cpu_mut().pc = state.cpu().pc + ops.a();
        } else {
            state.cpu_mut().pc = state.cpu().pc + Fp::ONE;
        }
        state.cpu_mut().operations.push(Operation::Beq);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for BneInstruction {
    const OPCODE: u32 = 6;

    fn execute(state: &mut M, ops: Operands) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.b();
        let cell_1 = state.mem_mut().read(clk, read_addr_1, true);
        let cell_2 = if ops.is_imm() == Fp::ONE {
            ops.c().into()
        } else {
            let read_addr_2 = state.cpu().fp + ops.c();
            state.mem_mut().read(clk, read_addr_2, true)
        };
        if cell_1 != cell_2 {
            state.cpu_mut().pc = state.cpu().pc + ops.a();
        } else {
            state.cpu_mut().pc = state.cpu().pc + Fp::ONE;
        }
        state.cpu_mut().operations.push(Operation::Bne);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for Imm32Instruction {
    const OPCODE: u32 = 7;

    fn execute(state: &mut M, ops: Operands) {
        let clk = state.cpu().clock;
        let write_addr = state.cpu().fp + ops.a();
        let value = Word::from([ops.b(), ops.c(), ops.d(), ops.e()]);
        state.mem_mut().write(clk, write_addr, value, true);
        state.cpu_mut().pc += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Imm32);
    }
}
