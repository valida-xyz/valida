#![no_std]

extern crate alloc;

use crate::columns::{CpuCols, CPU_COL_MAP, NUM_CPU_COLS};
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
    Beq(Option<Word<u8>> /*imm*/),
    Bne(Option<Word<u8>> /*imm*/),
    Imm32,
    Bus(Option<Word<u8>> /*imm*/),
    BusWithMemory(Option<Word<u8>> /*imm*/),
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
        let mut rows = self
            .operations
            .par_iter()
            .enumerate()
            .map(|(n, op)| self.op_to_row(n, op, machine))
            .collect::<Vec<_>>();

        // Set diff, diff_inv, and not_equal
        Self::compute_word_diffs(&mut rows);

        RowMajorMatrix::new(rows.concat(), NUM_CPU_COLS)
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let mem_sends = (0..3).map(|i| {
            let channel = &CPU_COL_MAP.mem_channels[i];
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

        let mut fields = vec![VirtualPairCol::single_main(CPU_COL_MAP.instruction.opcode)];
        fields.extend(
            CPU_COL_MAP
                .mem_channels
                .iter()
                .map(|c| c.value.into_iter().map(VirtualPairCol::single_main))
                .flatten()
                .collect::<Vec<_>>(),
        );
        fields.push(VirtualPairCol::single_main(
            CPU_COL_MAP.chip_channel.clk_or_zero,
        ));
        let send_general = Interaction {
            fields,
            count: VirtualPairCol::single_main(CPU_COL_MAP.opcode_flags.is_bus_op),
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
        cols.clk = F::from_canonical_usize(clk);

        self.set_memory_channel_values(clk, cols, machine);

        match op {
            Operation::Store32 => {
                cols.opcode_flags.is_store = F::ONE;
            }
            Operation::Load32 => {
                cols.opcode_flags.is_load = F::ONE;
            }
            Operation::Jal => {
                cols.opcode_flags.is_jal = F::ONE;
            }
            Operation::Jalv => {
                cols.opcode_flags.is_jalv = F::ONE;
            }
            Operation::Beq(imm) => {
                cols.opcode_flags.is_beq = F::ONE;
                self.set_imm_value(cols, *imm);
            }
            Operation::Bne(imm) => {
                cols.opcode_flags.is_bne = F::ONE;
                self.set_imm_value(cols, *imm);
            }
            Operation::Imm32 => {
                cols.opcode_flags.is_imm32 = F::ONE;
            }
            Operation::Bus(imm) => {
                cols.opcode_flags.is_bus_op = F::ONE;
                self.set_imm_value(cols, *imm);
            }
            Operation::BusWithMemory(imm) => {
                cols.opcode_flags.is_bus_op = F::ONE;
                cols.opcode_flags.is_bus_op_with_mem = F::ONE;
                self.set_imm_value(cols, *imm);
            }
        }

        row
    }

    fn set_memory_channel_values<F: PrimeField, M: MachineWithMemoryChip<F = F>>(
        &self,
        clk: usize,
        cols: &mut CpuCols<F>,
        machine: &M,
    ) {
        let memory = machine.mem();
        for ops in memory.operations.get(&(clk as u32)).iter() {
            let mut is_first_read = true;
            for op in ops.iter() {
                match op {
                    MemoryOperation::Read(addr, value) => {
                        if is_first_read {
                            cols.mem_channels[0].used = F::ONE;
                            cols.mem_channels[0].addr = F::from_canonical_u32(*addr);
                            cols.mem_channels[0].value = value.transform(F::from_canonical_u8);
                            is_first_read = false;
                        } else {
                            cols.mem_channels[1].used = F::ONE;
                            cols.mem_channels[1].addr = F::from_canonical_u32(*addr);
                            cols.mem_channels[1].value = value.transform(F::from_canonical_u8);
                        }
                    }
                    MemoryOperation::Write(addr, value) => {
                        cols.mem_channels[2].used = F::ONE;
                        cols.mem_channels[2].addr = F::from_canonical_u32(*addr);
                        cols.mem_channels[2].value = value.transform(F::from_canonical_u8);
                    }
                    _ => {}
                }
            }
        }
    }

    fn compute_word_diffs<F: PrimeField>(rows: &mut Vec<[F; NUM_CPU_COLS]>) {
        // Compute `diff`
        let mut diff = vec![F::ZERO; rows.len()];
        for n in 0..(rows.len() - 1) {
            let word_1 = CPU_COL_MAP.mem_channels[0]
                .value
                .into_iter()
                .map(|i| rows[n][i])
                .collect::<Vec<_>>();
            let word_2 = CPU_COL_MAP.mem_channels[1]
                .value
                .into_iter()
                .map(|i| rows[n][i])
                .collect::<Vec<_>>();
            for (a, b) in word_1.into_iter().zip(word_2) {
                diff[n] = (a - b) * (a - b);
            }
        }

        // Compute `diff_inv`
        // TODO: Implement inversion for Mersenne31 and uncomment line below
        //let diff_inv = batch_invert(diff.clone());
        let diff_inv = vec![F::ZERO; rows.len()];

        // Set trace values
        for n in 0..(rows.len() - 1) {
            rows[n][CPU_COL_MAP.diff] = diff[n];
            rows[n][CPU_COL_MAP.diff_inv] = diff_inv[n];
            if diff[n] != F::ZERO {
                rows[n][CPU_COL_MAP.not_equal] = F::ONE;
            }
        }
    }

    fn set_imm_value<F: PrimeField>(&self, cols: &mut CpuCols<F>, imm: Option<Word<u8>>) {
        if let Some(imm) = imm {
            cols.opcode_flags.is_imm_op = F::ONE;
            cols.mem_channels[1].value = imm.transform(F::from_canonical_u8);
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
        state.cpu_mut().push_op(Operation::Load32);
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
        state.cpu_mut().push_op(Operation::Store32);
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
        state.cpu_mut().push_op(Operation::Jal);
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
        state.cpu_mut().push_op(Operation::Jalv);
    }
}

impl<M> Instruction<M> for BeqInstruction
where
    M: MachineWithCpuChip,
{
    const OPCODE: u32 = 5;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let cell_1 = state.mem_mut().read(clk, read_addr_1, true);
        let cell_2 = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true)
        };
        if cell_1 == cell_2 {
            state.cpu_mut().pc = ops.a() as u32;
        } else {
            state.cpu_mut().pc = state.cpu().pc + 1;
        }
        state.cpu_mut().push_op(Operation::Beq(imm));
    }
}

impl<M> Instruction<M> for BneInstruction
where
    M: MachineWithCpuChip,
{
    const OPCODE: u32 = 6;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let cell_1 = state.mem_mut().read(clk, read_addr_1, true);
        let cell_2 = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true)
        };
        if cell_1 != cell_2 {
            state.cpu_mut().pc = ops.a() as u32;
        } else {
            state.cpu_mut().pc = state.cpu().pc + 1;
        }
        state.cpu_mut().push_op(Operation::Bne(imm));
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
        state.cpu_mut().push_op(Operation::Imm32);
    }
}

impl CpuChip {
    pub fn push_bus_op_with_memory(&mut self, imm: Option<Word<u8>>) {
        self.pc += 1;
        self.push_op(Operation::BusWithMemory(imm));
    }

    pub fn push_bus_op(&mut self, imm: Option<Word<u8>>) {
        self.pc += 1;
        self.push_op(Operation::Bus(imm));
    }

    pub fn push_op(&mut self, op: Operation) {
        self.operations.push(op);
        self.clock += 1;
        self.save_register_state();
    }

    pub fn save_register_state(&mut self) {
        let registers = Registers {
            pc: self.pc,
            fp: self.fp,
        };
        self.registers.push(registers);
    }
}
