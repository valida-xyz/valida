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

use p3_field::{AbstractField, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31 as Fp;
use valida_bus::{MachineWithGeneralBus, MachineWithMemBus};
use valida_machine::chip::Interaction;

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

#[derive(Default)]
pub struct CpuChip {
    pub clock: Fp,
    pub pc: Fp,
    pub fp: Fp,
    pub registers: Vec<Registers>,
    pub operations: Vec<Operation>,
}

#[derive(Default)]
pub struct Registers {
    pc: Fp,
    fp: Fp,
}

impl<M> Chip<M> for CpuChip
where
    M: MachineWithMemoryChip + MachineWithGeneralBus + MachineWithMemBus,
{
    type F = Fp;
    type FE = Fp; // FIXME

    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<Self::F> {
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
            let value = channel.value.0.map(|v| VirtualPairCol::single_main(v));
            let mut fields = vec![is_read, addr];
            fields.extend(value);
            Interaction {
                fields,
                count: VirtualPairCol::single_main(channel.used),
                argument_index: machine.mem_bus(),
            }
        });
        let send_general = Interaction {
            fields: vec![],
            count: VirtualPairCol::single_main(CPU_COL_INDICES.opcode_flags.is_bus_op),
            argument_index: 0,
        };
        mem_sends.chain(iter::once(send_general)).collect()
    }
}

impl CpuChip {
    fn op_to_row<M>(&self, clk: usize, op: Operation, machine: &M) -> [Fp; NUM_CPU_COLS]
    where
        M: MachineWithMemoryChip,
    {
        let mut cols = CpuCols::default();
        cols.pc = self.registers[clk].pc;
        cols.fp = self.registers[clk].fp;

        self.set_memory_trace_values(clk, &mut cols, machine);

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
                cols.chip_channel.opcode = Fp::from_canonical_u32(opcode);
                // TODO: Set other chip channel fields in an additional trace pass,
                // or read this information from the machine and set it here?
            }
        }

        let row: [Fp; NUM_CPU_COLS] = unsafe { transmute(cols) };
        row
    }

    fn set_memory_trace_values<M: MachineWithMemoryChip>(
        &self,
        _clk: usize,
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
        let cell = state.mem_mut().read(clk, read_addr, true).into();
        state.cpu_mut().fp += cell;
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
            state.cpu_mut().pc = ops.a();
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
            state.cpu_mut().pc = ops.a();
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
