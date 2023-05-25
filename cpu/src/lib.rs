#![no_std]

extern crate alloc;

use crate::columns::{CpuCols, CPU_COL_INDICES, NUM_CPU_COLS};
use alloc::vec;
use alloc::vec::Vec;
use core::iter;
use core::marker::Sync;
use core::mem::transmute;
use valida_bus::{MachineWithGeneralBus, MachineWithMemBus};
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};
use valida_memory::{MachineWithMemoryChip, Operation as MemoryOperation};

use p3_air::VirtualPairCol;
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

pub mod columns;
pub mod stark;

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

#[derive(Default)]
pub struct CpuChip {
    pub clock: u32,
    pub pc: u32,
    pub fp: u32,
    pub registers: Vec<Registers>,
    pub operations: Vec<Operation>,
}

#[derive(Default)]
pub struct Registers {
    pc: u32,
    fp: u32,
}

impl<M> Chip<M> for CpuChip
where
    M: MachineWithMemoryChip + MachineWithGeneralBus + MachineWithMemBus + Sync,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .par_iter()
            .enumerate()
            .map(|(n, op)| self.op_to_row(n, &op, machine))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_CPU_COLS)
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let mem_sends = (0..3).map(|i| {
            let channel = &CPU_COL_INDICES.mem_channels[i];
            let is_read = VirtualPairCol::single_main(channel.is_read);
            let addr = VirtualPairCol::single_main(channel.addr);
            let value = channel.value.0.map(VirtualPairCol::single_main);

            let mut fields = vec![is_read, addr];
            fields.extend(value);

            Interaction {
                fields,
                count: VirtualPairCol::single_main(channel.used),
                argument_index: machine.mem_bus(),
            }
        });

        let send_general = Interaction {
            fields: CPU_COL_INDICES
                .chip_channel
                .iter_flat()
                .map(VirtualPairCol::single_main)
                .collect(),
            count: VirtualPairCol::single_main(CPU_COL_INDICES.opcode_flags.is_bus_op),
            argument_index: machine.general_bus(),
        };

        mem_sends.chain(iter::once(send_general)).collect()
    }
}

impl CpuChip {
    fn op_to_row<F: PrimeField, M: MachineWithMemoryChip<F = F>>(
        &self,
        clk: usize,
        op: &Operation,
        machine: &M,
    ) -> [F; NUM_CPU_COLS]
    where
        M: MachineWithMemoryChip,
    {
        let mut row = [F::ZERO; NUM_CPU_COLS];
        let mut cols: &mut CpuCols<F> = unsafe { transmute(&mut row) };

        cols.pc = F::from_canonical_u32(self.registers[clk].pc);
        cols.fp = F::from_canonical_u32(self.registers[clk].fp);

        self.set_memory_trace_values(clk, cols, machine);

        match op {
            Operation::Store32 => {}
            Operation::Load32 => {}
            Operation::Jal => {}
            Operation::Jalv => {}
            Operation::Beq => {
                cols.opcode_flags.is_beq = F::ONE;
            }
            Operation::Bne => {}
            Operation::Imm32 => {
                cols.opcode_flags.is_imm32 = F::ONE;
            }
            Operation::Bus(opcode) => {
                cols.opcode_flags.is_bus_op = F::ONE;
                cols.chip_channel.opcode = F::from_canonical_u32(*opcode);
                // TODO: Set other chip channel fields in an additional trace pass,
                // or read this information from the machine and set it here?
            }
        }

        row
    }

    fn set_memory_trace_values<F: PrimeField, M: MachineWithMemoryChip<F = F>>(
        &self,
        _clk: usize,
        cols: &mut CpuCols<F>,
        machine: &M,
    ) {
        let memory = machine.mem();
        for (_, ops) in memory.operations.iter() {
            let mut is_first_read = true;
            for op in ops {
                match op {
                    MemoryOperation::Read(addr, value) => {
                        if is_first_read {
                            cols.mem_channels[0].used = F::ONE;
                            cols.mem_channels[0].addr = F::from_canonical_u32(*addr);
                            cols.mem_channels[0].value = value.to_field();
                            is_first_read = false;
                        } else {
                            cols.mem_channels[1].used = F::ONE;
                            cols.mem_channels[1].addr = F::from_canonical_u32(*addr);
                            cols.mem_channels[1].value = value.to_field();
                        }
                    }
                    MemoryOperation::Write(addr, value) => {
                        cols.mem_channels[2].used = F::ONE;
                        cols.mem_channels[2].addr = F::from_canonical_u32(*addr);
                        cols.mem_channels[2].value = value.to_field();
                    }
                    _ => {}
                }
            }
        }
    }
}

pub trait MachineWithCpuChip: MachineWithMemoryChip {
    fn cpu(&self) -> &CpuChip;
    fn cpu_mut(&mut self) -> &mut CpuChip;
}

instructions!(
    Load32Instruction,
    Store32Instruction,
    JalInstruction,
    JalvInstruction,
    BeqInstruction,
    BneInstruction,
    Imm32Instruction
);

impl<M> Instruction<M> for Load32Instruction
where
    M: MachineWithCpuChip,
{
    const OPCODE: u32 = 1;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let read_addr_1 = (state.cpu().fp as i32 + ops.c()) as u32;
        let read_addr_2 = state.mem_mut().read(clk, read_addr_1, true);
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let cell = state.mem_mut().read(clk, read_addr_2.into(), true);
        state.mem_mut().write(clk, write_addr, cell, true);
        state.cpu_mut().pc += 1;
        state.cpu_mut().clock += 1;
        state.cpu_mut().operations.push(Operation::Load32);
        set_pc_and_fp(state);
    }
}

impl<M> Instruction<M> for Store32Instruction
where
    M: MachineWithCpuChip,
{
    const OPCODE: u32 = 2;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let read_addr = (state.cpu().fp as i32 + ops.c()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.b()) as u32;
        let cell = state.mem_mut().read(clk, read_addr, true);
        state.mem_mut().write(clk, write_addr, cell, true);
        state.cpu_mut().pc += 1;
        state.cpu_mut().clock += 1;
        state.cpu_mut().operations.push(Operation::Store32);
        set_pc_and_fp(state);
    }
}

impl<M> Instruction<M> for JalInstruction
where
    M: MachineWithCpuChip,
{
    const OPCODE: u32 = 3;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        // Store pc + 1 to local stack variable at offset a
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let next_pc = state.cpu().pc + 1;
        state.mem_mut().write(clk, write_addr, next_pc.into(), true);
        // Set pc to the field element b
        state.cpu_mut().pc = ops.b() as u32;
        // Set fp to fp + c
        state.cpu_mut().fp = (state.cpu().fp as i32 + ops.c()) as u32;
        state.cpu_mut().clock += 1;
        state.cpu_mut().operations.push(Operation::Jal);
        set_pc_and_fp(state);
    }
}

impl<M> Instruction<M> for JalvInstruction
where
    M: MachineWithCpuChip,
{
    const OPCODE: u32 = 4;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        // Store pc + 1 to local stack variable at offset a
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let next_pc = state.cpu().pc + 1;
        state.mem_mut().write(clk, write_addr, next_pc.into(), true);
        // Set pc to the field element [b]
        let read_addr = (state.cpu().fp as i32 + ops.b()) as u32;
        state.cpu_mut().pc = state.mem_mut().read(clk, read_addr, true).into();
        // Set fp to [c]
        let read_addr = (state.cpu().fp as i32 + ops.c()) as u32;
        let cell: u32 = state.mem_mut().read(clk, read_addr, true).into();
        state.cpu_mut().fp += cell;
        state.cpu_mut().clock += 1;
        state.cpu_mut().operations.push(Operation::Jalv);
        set_pc_and_fp(state);
    }
}

impl<M> Instruction<M> for BeqInstruction
where
    M: MachineWithCpuChip,
{
    const OPCODE: u32 = 5;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let cell_1 = state.mem_mut().read(clk, read_addr_1, true);
        let cell_2 = if ops.is_imm() == 1 {
            (ops.c() as u32).into()
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true)
        };
        if cell_1 == cell_2 {
            state.cpu_mut().pc = ops.a() as u32;
        } else {
            state.cpu_mut().pc = state.cpu().pc + 1;
        }
        state.cpu_mut().clock += 1;
        state.cpu_mut().operations.push(Operation::Beq);
        set_pc_and_fp(state);
    }
}

impl<M> Instruction<M> for BneInstruction
where
    M: MachineWithCpuChip,
{
    const OPCODE: u32 = 6;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let cell_1 = state.mem_mut().read(clk, read_addr_1, true);
        let cell_2 = if ops.is_imm() == 1 {
            (ops.c() as u32).into()
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true)
        };
        if cell_1 != cell_2 {
            state.cpu_mut().pc = ops.a() as u32;
        } else {
            state.cpu_mut().pc = state.cpu().pc + 1;
        }
        state.cpu_mut().clock += 1;
        state.cpu_mut().operations.push(Operation::Bne);
        set_pc_and_fp(state);
    }
}

impl<M> Instruction<M> for Imm32Instruction
where
    M: MachineWithCpuChip,
{
    const OPCODE: u32 = 7;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let value = Word([ops.b() as u8, ops.c() as u8, ops.d() as u8, ops.e() as u8]);
        state.mem_mut().write(clk, write_addr, value.into(), true);
        state.cpu_mut().pc += 1;
        state.cpu_mut().clock += 1;
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
