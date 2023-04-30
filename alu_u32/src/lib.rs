#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use p3_field::field::Field;
use p3_mersenne_31::Mersenne31 as Fp;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{Instruction, Operands, Word, MEMORY_CELL_BYTES};

pub mod columns;
mod stark;

pub enum Operation {
    Add32,
    Mul32,
}

pub struct ALU32Chip {
    pub clock: Fp,
    pub operations: Vec<Operation>,
}

pub trait MachineWithALU32Chip: MachineWithCpuChip {
    fn alu_u32(&self) -> &ALU32Chip;
    fn alu_u32_mut(&mut self) -> &mut ALU32Chip;
}

pub struct Add32Instruction;
pub struct Mul32Instruction;

impl<M: MachineWithALU32Chip> Instruction<M> for Add32Instruction {
    const OPCODE: u32 = 8;

    fn execute(state: &mut M, ops: Operands) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.b();
        let read_addr_2 = state.cpu().fp + ops.c();
        let write_addr = state.cpu().fp + ops.a();
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c = state.mem_mut().read(clk, read_addr_2, true);

        // FIXME
        let mut a = Word::<Fp>::default();
        let mut carry = 0u8;
        for i in (0..MEMORY_CELL_BYTES).rev() {
            let b_i = b[i].as_canonical_u32() as u8;
            let c_i = c[i].as_canonical_u32() as u8;
            let (sum, overflow) = b_i.overflowing_add(c_i);
            let (sum_with_carry, carry_overflow) = sum.overflowing_add(carry);
            carry = overflow as u8 + carry_overflow as u8;
            a[i] = Fp::from(sum_with_carry as u32);
        }
        state.mem_mut().write(clk, write_addr, a, true);

        state.alu_u32_mut().operations.push(Operation::Add32);
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().pc += Fp::ONE;
        // TODO: Set register log in the CPU as well
    }
}

impl<M: MachineWithALU32Chip> Instruction<M> for Mul32Instruction {
    const OPCODE: u32 = 9;

    fn execute(state: &mut M, ops: Operands) {
        todo!()
    }
}
