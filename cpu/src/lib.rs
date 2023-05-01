#![no_std]

extern crate alloc;

use crate::columns::{CpuCols, NUM_CPU_COLS};
use alloc::vec::Vec;
use core::mem::transmute;
use p3_field::field::Field;
use p3_mersenne_31::Mersenne31 as Fp;
use valida_machine::{trace::TraceGenerator, Instruction, Machine, Operands, Word};
use valida_memory::{MachineWithMemoryChip, Operation as MemoryOperation};

pub mod columns;
mod stark;

#[derive(Clone)]
pub enum Operation {
    Store32,
    Load32,
    Jal,
    Jalv,
    Beq,
    Bne,
    Imm32,
    Bus(u32),
}

pub struct CpuChip {
    pub clock: Fp,
    pub pc: Fp,
    pub fp: Fp,
    pub registers: Vec<Registers>,
    pub operations: Vec<Operation>,
}

pub struct Registers {
    pc: Fp,
    fp: Fp,
}

impl<M> TraceGenerator<M, Fp> for CpuChip
where
    M: MachineWithMemoryChip,
{
    const NUM_COLS: usize = NUM_CPU_COLS;
    type Operation = Operation;

    fn generate_trace(&self, machine: &M) -> Vec<[Fp; NUM_CPU_COLS]> {
        let rows = self
            .operations
            .iter() // TODO: Parallelize with rayon
            .cloned()
            .enumerate()
            .map(|(n, op)| self.op_to_row(n, op, machine))
            .collect();
        rows
    }

    fn op_to_row(&self, n: usize, op: Operation, machine: &M) -> [Fp; NUM_CPU_COLS] {
        let mut cols = CpuCols::default();
        cols.pc = self.registers[n].pc;
        cols.fp = self.registers[n].fp;

        self.set_memory_trace_values(n, &mut cols, machine);

        match op {
            Operation::Store32 => {}
            Operation::Load32 => {}
            Operation::Jal => {}
            Operation::Jalv => {}
            Operation::Beq => {
                cols.opcode_flags.is_beq = Fp::ONE;
            }
            Operation::Bne => {}
            Operation::Imm32 => {
                cols.opcode_flags.is_imm32 = Fp::ONE;
            }
            Operation::Bus(opcode) => {
                cols.opcode_flags.is_bus_op = Fp::ONE;
                cols.chip_channel.opcode = opcode.into();
                // TODO: Set other chip channel fields in an additional trace pass,
                // or read this information from the machine and set it here?
            }
        }

        let row: [Fp; NUM_CPU_COLS] = unsafe { transmute(cols) };
        row
    }
}

impl CpuChip {
    fn set_memory_trace_values<M: MachineWithMemoryChip>(
        &self,
        n: usize,
        cols: &mut CpuCols<Fp>,
        machine: &M,
    ) {
        let memory = machine.mem();
        for (_, ops) in memory.operations.iter() {
            let mut is_first_read = true;
            for op in ops {
                match op {
                    MemoryOperation::Read(addr, value) => {
                        if is_first_read {
                            cols.mem_channels[0].used = Fp::ONE;
                            cols.mem_channels[0].addr = *addr;
                            cols.mem_channels[0].value = *value;
                            is_first_read = false;
                        } else {
                            cols.mem_channels[1].used = Fp::ONE;
                            cols.mem_channels[1].addr = *addr;
                            cols.mem_channels[1].value = *value;
                        }
                    }
                    MemoryOperation::Write(addr, value) => {
                        cols.mem_channels[2].used = Fp::ONE;
                        cols.mem_channels[2].addr = *addr;
                        cols.mem_channels[2].value = *value;
                    }
                }
            }
        }
    }
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

    fn execute(state: &mut M, ops: Operands<Fp>) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.c();
        let read_addr_2 = state.mem_mut().read(clk, read_addr_1, true);
        let write_addr = state.cpu().fp + ops.a();
        let cell = state.mem_mut().read(clk, read_addr_2, true);
        state.mem_mut().write(clk, write_addr, cell, true);
        state.cpu_mut().pc += Fp::ONE;
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Load32);
        set_pc_and_fp(state);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for Store32Instruction {
    const OPCODE: u32 = 2;

    fn execute(state: &mut M, ops: Operands<Fp>) {
        let clk = state.cpu().clock;
        let read_addr = state.cpu().fp + ops.c();
        let write_addr = state.cpu().fp + ops.b();
        let cell = state.mem_mut().read(clk, read_addr, true);
        state.mem_mut().write(clk, write_addr, cell, true);
        state.cpu_mut().pc += Fp::ONE;
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Store32);
        set_pc_and_fp(state);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for JalInstruction {
    const OPCODE: u32 = 3;

    fn execute(state: &mut M, ops: Operands<Fp>) {
        let clk = state.cpu().clock;
        // Store pc + 1 to local stack variable at offset a
        let write_addr = state.cpu().fp + ops.a();
        let next_pc = state.cpu().pc + Fp::ONE;
        state.mem_mut().write(clk, write_addr, next_pc, true);
        // Set pc to the field element b
        state.cpu_mut().pc = ops.b();
        // Set fp to fp + c
        state.cpu_mut().fp += ops.c();
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Jal);
        set_pc_and_fp(state);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for JalvInstruction {
    const OPCODE: u32 = 4;

    fn execute(state: &mut M, ops: Operands<Fp>) {
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
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Jalv);
        set_pc_and_fp(state);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for BeqInstruction {
    const OPCODE: u32 = 5;

    fn execute(state: &mut M, ops: Operands<Fp>) {
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
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Beq);
        set_pc_and_fp(state);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for BneInstruction {
    const OPCODE: u32 = 6;

    fn execute(state: &mut M, ops: Operands<Fp>) {
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
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Bne);
        set_pc_and_fp(state);
    }
}

impl<M: MachineWithCpuChip> Instruction<M> for Imm32Instruction {
    const OPCODE: u32 = 7;

    fn execute(state: &mut M, ops: Operands<Fp>) {
        let clk = state.cpu().clock;
        let write_addr = state.cpu().fp + ops.a();
        let value = Word::from([ops.b(), ops.c(), ops.d(), ops.e()]);
        state.mem_mut().write(clk, write_addr, value, true);
        state.cpu_mut().pc += Fp::ONE;
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().operations.push(Operation::Imm32);
        set_pc_and_fp(state);
    }
}

fn set_pc_and_fp(state: &mut impl MachineWithCpuChip) {
    let registers = Registers {
        pc: state.cpu().pc,
        fp: state.cpu().fp,
    };
    state.cpu_mut().registers.push(registers);
}
