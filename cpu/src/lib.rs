#![no_std]

extern crate alloc;

use crate::columns::{CpuCols, CPU_COL_INDICES, NUM_CPU_COLS};
use alloc::vec;
use alloc::vec::Vec;
use core::iter;
use core::mem::transmute;
use p3_air::VirtualPairCol;
use valida_machine::{instructions, Chip, Instruction, Operands, Word};
use valida_memory::{MachineWithMemoryChip, Operation as MemoryOperation};

use p3_field::{AbstractField, PrimeField, PrimeField64};
use p3_matrix::dense::RowMajorMatrix;
use valida_bus::{MachineWithGeneralBus, MachineWithMemBus};
use valida_machine::chip::Interaction;

pub mod columns;
mod stark;

#[derive(Clone)]
pub enum Operation<F> {
    Store32,
    Load32,
    Jal,
    Jalv,
    Beq,
    Bne,
    Imm32,
    Bus(F),
}

#[derive(Default)]
pub struct CpuChip<F> {
    pub clock: F,
    pub pc: F,
    pub fp: F,
    pub registers: Vec<Registers<F>>,
    pub operations: Vec<Operation<F>>,
}

#[derive(Default)]
pub struct Registers<F> {
    pc: F,
    fp: F,
}

impl<M> Chip<M> for CpuChip<M::F>
where
    M: MachineWithMemoryChip + MachineWithGeneralBus + MachineWithMemBus,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .iter()
            .cloned()
            .enumerate()
            .map(|(n, op)| self.op_to_row(n, op, machine))
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

impl<F: PrimeField> CpuChip<F> {
    fn op_to_row<M: MachineWithMemoryChip<F = F>>(
        &self,
        clk: usize,
        op: Operation<F>,
        machine: &M,
    ) -> [F; NUM_CPU_COLS]
    where
        M: MachineWithMemoryChip,
    {
        let mut row = [F::ZERO; NUM_CPU_COLS];
        let mut cols: &mut CpuCols<F> = unsafe { transmute(&mut row) };

        cols.pc = self.registers[clk].pc;
        cols.fp = self.registers[clk].fp;

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
                cols.chip_channel.opcode = opcode;
                // TODO: Set other chip channel fields in an additional trace pass,
                // or read this information from the machine and set it here?
            }
        }

        row
    }

    fn set_memory_trace_values<M: MachineWithMemoryChip<F = F>>(
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
                            cols.mem_channels[0].addr = *addr;
                            cols.mem_channels[0].value = *value;
                            is_first_read = false;
                        } else {
                            cols.mem_channels[1].used = F::ONE;
                            cols.mem_channels[1].addr = *addr;
                            cols.mem_channels[1].value = *value;
                        }
                    }
                    MemoryOperation::Write(addr, value) => {
                        cols.mem_channels[2].used = F::ONE;
                        cols.mem_channels[2].addr = *addr;
                        cols.mem_channels[2].value = *value;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub trait MachineWithCpuChip: MachineWithMemoryChip {
    fn cpu(&self) -> &CpuChip<Self::F>;
    fn cpu_mut(&mut self) -> &mut CpuChip<Self::F>;
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

impl<F, M> Instruction<M> for Load32Instruction
where
    F: PrimeField,
    M: MachineWithCpuChip<F = F>,
{
    const OPCODE: u32 = 1;

    fn execute(state: &mut M, ops: Operands<F>) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.c();
        let read_addr_2 = state.mem_mut().read(clk, read_addr_1, true);
        let write_addr = state.cpu().fp + ops.a();
        let cell = state.mem_mut().read(clk, read_addr_2.to_value(), true);
        state.mem_mut().write(clk, write_addr, cell, true);
        state.cpu_mut().pc += F::ONE;
        state.cpu_mut().clock += F::ONE;
        state.cpu_mut().operations.push(Operation::Load32);
        set_pc_and_fp(state);
    }
}

impl<F, M> Instruction<M> for Store32Instruction
where
    F: PrimeField,
    M: MachineWithCpuChip<F = F>,
{
    const OPCODE: u32 = 2;

    fn execute(state: &mut M, ops: Operands<F>) {
        let clk = state.cpu().clock;
        let read_addr = state.cpu().fp + ops.c();
        let write_addr = state.cpu().fp + ops.b();
        let cell = state.mem_mut().read(clk, read_addr, true);
        state.mem_mut().write(clk, write_addr, cell, true);
        state.cpu_mut().pc += F::ONE;
        state.cpu_mut().clock += F::ONE;
        state.cpu_mut().operations.push(Operation::Store32);
        set_pc_and_fp(state);
    }
}

impl<F, M> Instruction<M> for JalInstruction
where
    F: PrimeField,
    M: MachineWithCpuChip<F = F>,
{
    const OPCODE: u32 = 3;

    fn execute(state: &mut M, ops: Operands<F>) {
        let clk = state.cpu().clock;
        // Store pc + 1 to local stack variable at offset a
        let write_addr = state.cpu().fp + ops.a();
        let next_pc = state.cpu().pc + F::ONE;
        state.mem_mut().write(clk, write_addr, next_pc, true);
        // Set pc to the field element b
        state.cpu_mut().pc = ops.b();
        // Set fp to fp + c
        state.cpu_mut().fp += ops.c();
        state.cpu_mut().clock += F::ONE;
        state.cpu_mut().operations.push(Operation::Jal);
        set_pc_and_fp(state);
    }
}

impl<F, M> Instruction<M> for JalvInstruction
where
    F: PrimeField,
    M: MachineWithCpuChip<F = F>,
{
    const OPCODE: u32 = 4;

    fn execute(state: &mut M, ops: Operands<F>) {
        let clk = state.cpu().clock;
        // Store pc + 1 to local stack variable at offset a
        let write_addr = state.cpu().fp + ops.a();
        let next_pc = state.cpu().pc + F::ONE;
        state.mem_mut().write(clk, write_addr, next_pc, true);
        // Set pc to the field element [b]
        let read_addr = state.cpu().fp + ops.b();
        state.cpu_mut().pc = state.mem_mut().read(clk, read_addr, true).to_value();
        // Set fp to [c]
        let read_addr = state.cpu().fp + ops.c();
        let cell = state.mem_mut().read(clk, read_addr, true).to_value();
        state.cpu_mut().fp += cell;
        state.cpu_mut().clock += F::ONE;
        state.cpu_mut().operations.push(Operation::Jalv);
        set_pc_and_fp(state);
    }
}

impl<F, M> Instruction<M> for BeqInstruction
where
    F: PrimeField,
    M: MachineWithCpuChip<F = F>,
{
    const OPCODE: u32 = 5;

    fn execute(state: &mut M, ops: Operands<F>) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.b();
        let cell_1 = state.mem_mut().read(clk, read_addr_1, true);
        let cell_2 = if ops.is_imm() == F::ONE {
            ops.c().into()
        } else {
            let read_addr_2 = state.cpu().fp + ops.c();
            state.mem_mut().read(clk, read_addr_2, true)
        };
        if cell_1 == cell_2 {
            state.cpu_mut().pc = ops.a();
        } else {
            state.cpu_mut().pc = state.cpu().pc + F::ONE;
        }
        state.cpu_mut().clock += F::ONE;
        state.cpu_mut().operations.push(Operation::Beq);
        set_pc_and_fp(state);
    }
}

impl<F, M> Instruction<M> for BneInstruction
where
    F: PrimeField,
    M: MachineWithCpuChip<F = F>,
{
    const OPCODE: u32 = 6;

    fn execute(state: &mut M, ops: Operands<F>) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.b();
        let cell_1 = state.mem_mut().read(clk, read_addr_1, true);
        let cell_2 = if ops.is_imm() == F::ONE {
            ops.c().into()
        } else {
            let read_addr_2 = state.cpu().fp + ops.c();
            state.mem_mut().read(clk, read_addr_2, true)
        };
        if cell_1 != cell_2 {
            state.cpu_mut().pc = ops.a();
        } else {
            state.cpu_mut().pc = state.cpu().pc + F::ONE;
        }
        state.cpu_mut().clock += F::ONE;
        state.cpu_mut().operations.push(Operation::Bne);
        set_pc_and_fp(state);
    }
}

impl<F, M> Instruction<M> for Imm32Instruction
where
    F: PrimeField,
    M: MachineWithCpuChip<F = F>,
{
    const OPCODE: u32 = 7;

    fn execute(state: &mut M, ops: Operands<F>) {
        let clk = state.cpu().clock;
        let write_addr = state.cpu().fp + ops.a();
        let value = Word([ops.b(), ops.c(), ops.d(), ops.e()]);
        state.mem_mut().write(clk, write_addr, value, true);
        state.cpu_mut().pc += F::ONE;
        state.cpu_mut().clock += F::ONE;
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
