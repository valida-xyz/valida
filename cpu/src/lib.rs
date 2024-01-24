#![no_std]

extern crate alloc;

use crate::columns::{CpuCols, CPU_COL_MAP, NUM_CPU_COLS};
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::iter;
use core::marker::Sync;
use core::mem::transmute;
use valida_bus::{MachineWithGeneralBus, MachineWithMemBus, MachineWithProgramBus};
use valida_machine::{
    instructions, AdviceProvider, Chip, Instruction, InstructionWord, Interaction, Operands, Word,
};
use valida_memory::{MachineWithMemoryChip, Operation as MemoryOperation};
use valida_opcodes::{
    BEQ, BNE, BYTES_PER_INSTR, IMM32, JAL, JALV, LOAD32, READ_ADVICE, STOP, STORE32,
};

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::*;
use valida_machine::StarkConfig;
use valida_util::batch_multiplicative_inverse_allowing_zero;

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
    ReadAdvice,
    Stop,
}

#[derive(Default)]
pub struct CpuChip {
    pub clock: u32,
    pub pc: u32,
    pub fp: u32,
    pub registers: Vec<Registers>,
    pub operations: Vec<Operation>,
    pub instructions: Vec<InstructionWord<i32>>,
}

#[derive(Default)]
pub struct Registers {
    pc: u32,
    fp: u32,
}

impl<M, SC> Chip<M, SC> for CpuChip
where
    M: MachineWithProgramBus<SC::Val>
        + MachineWithMemoryChip<SC::Val>
        + MachineWithGeneralBus<SC::Val>
        + MachineWithMemBus<SC::Val>
        + Sync,
    SC: StarkConfig,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<SC::Val> {
        let mut rows = self
            .operations
            .as_slice()
            .into_par_iter()
            .enumerate()
            .map(|(n, op)| self.op_to_row::<M, SC>(n, op, machine))
            .collect::<Vec<_>>();

        // Set diff, diff_inv, and not_equal
        Self::compute_word_diffs(&mut rows);

        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_CPU_COLS);

        Self::pad_to_power_of_two(&mut trace.values);

        trace
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        // Memory bus channels
        let mem_sends = (0..3).map(|i| {
            let channel = &CPU_COL_MAP.mem_channels[i];
            let is_read = VirtualPairCol::single_main(channel.is_read);
            let clk = VirtualPairCol::single_main(CPU_COL_MAP.clk);
            let addr = VirtualPairCol::single_main(channel.addr);
            let value = channel.value.0.map(VirtualPairCol::single_main);

            let mut fields = vec![is_read, clk, addr];
            fields.extend(value);

            Interaction {
                fields,
                count: VirtualPairCol::single_main(channel.used),
                argument_index: machine.mem_bus(),
            }
        });

        // General bus channel
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

        // Program ROM bus channel
        let pc = VirtualPairCol::single_main(CPU_COL_MAP.pc);
        let opcode = VirtualPairCol::single_main(CPU_COL_MAP.instruction.opcode);
        let mut fields = vec![pc, opcode];
        fields.extend(
            CPU_COL_MAP
                .instruction
                .operands
                .0
                .map(|op| VirtualPairCol::single_main(op)),
        );
        let send_program = Interaction {
            fields,
            count: VirtualPairCol::one(),
            argument_index: machine.program_bus(),
        };

        mem_sends
            .chain(iter::once(send_general))
            .chain(iter::once(send_program))
            .collect()
    }
}

impl CpuChip {
    fn op_to_row<M, SC>(&self, clk: usize, op: &Operation, machine: &M) -> [SC::Val; NUM_CPU_COLS]
    where
        M: MachineWithMemoryChip<SC::Val>,
        SC: StarkConfig,
    {
        let mut row = [SC::Val::zero(); NUM_CPU_COLS];
        let cols: &mut CpuCols<SC::Val> = unsafe { transmute(&mut row) };

        cols.pc = SC::Val::from_canonical_u32(self.registers[clk].pc);
        cols.fp = SC::Val::from_canonical_u32(self.registers[clk].fp);
        cols.clk = SC::Val::from_canonical_usize(clk);

        self.set_instruction_values(clk, cols);
        self.set_memory_channel_values::<M, SC>(clk, cols, machine);

        match op {
            Operation::Store32 => {
                cols.opcode_flags.is_store = SC::Val::one();
            }
            Operation::Load32 => {
                cols.opcode_flags.is_load = SC::Val::one();
            }
            Operation::Jal => {
                cols.opcode_flags.is_jal = SC::Val::one();
            }
            Operation::Jalv => {
                cols.opcode_flags.is_jalv = SC::Val::one();
            }
            Operation::Beq(imm) => {
                cols.opcode_flags.is_beq = SC::Val::one();
                self.set_imm_value(cols, *imm);
            }
            Operation::Bne(imm) => {
                cols.opcode_flags.is_bne = SC::Val::one();
                self.set_imm_value(cols, *imm);
            }
            Operation::Imm32 => {
                cols.opcode_flags.is_imm32 = SC::Val::one();
            }
            Operation::Bus(imm) => {
                cols.opcode_flags.is_bus_op = SC::Val::one();
                self.set_imm_value(cols, *imm);
            }
            Operation::BusWithMemory(imm) => {
                cols.opcode_flags.is_bus_op = SC::Val::one();
                cols.opcode_flags.is_bus_op_with_mem = SC::Val::one();
                self.set_imm_value(cols, *imm);
            }
            Operation::ReadAdvice => {
                cols.opcode_flags.is_advice = SC::Val::one();
            }
            Operation::Stop => {
                cols.opcode_flags.is_stop = SC::Val::one();
            }
        }

        row
    }

    fn set_instruction_values<F: Field>(&self, clk: usize, cols: &mut CpuCols<F>) {
        cols.instruction.opcode = F::from_canonical_u32(self.instructions[clk].opcode);
        cols.instruction.operands =
            Operands::<F>::from_i32_slice(&self.instructions[clk].operands.0);
    }

    fn set_memory_channel_values<M: MachineWithMemoryChip<SC::Val>, SC: StarkConfig>(
        &self,
        clk: usize,
        cols: &mut CpuCols<SC::Val>,
        machine: &M,
    ) {
        cols.mem_channels[0].is_read = SC::Val::one();
        cols.mem_channels[1].is_read = SC::Val::one();
        cols.mem_channels[2].is_read = SC::Val::zero();

        let memory = machine.mem();
        for ops in memory.operations.get(&(clk as u32)).iter() {
            let mut is_first_read = true;
            for op in ops.iter() {
                match op {
                    MemoryOperation::Read(addr, value) => {
                        if is_first_read {
                            cols.mem_channels[0].used = SC::Val::one();
                            cols.mem_channels[0].addr = SC::Val::from_canonical_u32(*addr);
                            cols.mem_channels[0].value =
                                value.transform(SC::Val::from_canonical_u8);
                            is_first_read = false;
                        } else {
                            cols.mem_channels[1].used = SC::Val::one();
                            cols.mem_channels[1].addr = SC::Val::from_canonical_u32(*addr);
                            cols.mem_channels[1].value =
                                value.transform(SC::Val::from_canonical_u8);
                        }
                    }
                    MemoryOperation::Write(addr, value) => {
                        cols.mem_channels[2].used = SC::Val::one();
                        cols.mem_channels[2].addr = SC::Val::from_canonical_u32(*addr);
                        cols.mem_channels[2].value = value.transform(SC::Val::from_canonical_u8);
                    }
                    _ => {}
                }
            }
        }
    }

    fn compute_word_diffs<F: PrimeField>(rows: &mut Vec<[F; NUM_CPU_COLS]>) {
        // Compute `diff`
        let mut diff = vec![F::zero(); rows.len()];
        for i in 0..rows.len() {
            let word_1 = CPU_COL_MAP.mem_channels[0]
                .value
                .into_iter()
                .map(|i| rows[i][i])
                .collect::<Vec<_>>();
            let word_2 = CPU_COL_MAP.mem_channels[1]
                .value
                .into_iter()
                .map(|i| rows[i][i])
                .collect::<Vec<_>>();
            for (a, b) in word_1.into_iter().zip(word_2) {
                diff[i] += (a - b).square();
            }
        }

        // Compute `diff_inv`
        let diff_inv = batch_multiplicative_inverse_allowing_zero(diff.clone());

        // Set trace values
        for i in 0..rows.len() {
            rows[i][CPU_COL_MAP.diff] = diff[i];
            rows[i][CPU_COL_MAP.diff_inv] = diff_inv[i];
            if diff[i] != F::zero() {
                rows[i][CPU_COL_MAP.not_equal] = F::one();
            }
        }
    }

    fn pad_to_power_of_two<F: PrimeField>(values: &mut Vec<F>) {
        let len = values.len();
        let n_real_rows = values.len() / NUM_CPU_COLS;

        let last_row = &values[len - NUM_CPU_COLS..];
        let pc = last_row[CPU_COL_MAP.pc];
        let fp = last_row[CPU_COL_MAP.fp];
        let clk = last_row[CPU_COL_MAP.clk];

        values.resize(n_real_rows.next_power_of_two() * NUM_CPU_COLS, F::zero());

        // Interpret values as a slice of arrays of length `NUM_CPU_COLS`
        let rows = unsafe {
            core::slice::from_raw_parts_mut(
                values.as_mut_ptr() as *mut [F; NUM_CPU_COLS],
                values.len() / NUM_CPU_COLS,
            )
        };

        rows[n_real_rows..]
            .par_iter_mut()
            .enumerate()
            .for_each(|(n, padded_row)| {
                padded_row[CPU_COL_MAP.pc] = pc;
                padded_row[CPU_COL_MAP.fp] = fp;
                padded_row[CPU_COL_MAP.clk] = clk + F::from_canonical_u32(n as u32 + 1);

                // STOP instructions
                padded_row[CPU_COL_MAP.opcode_flags.is_stop] = F::one();
                padded_row[CPU_COL_MAP.instruction.opcode] = F::from_canonical_u32(STOP);

                // Memory columns
                padded_row[CPU_COL_MAP.mem_channels[0].is_read] = F::one();
                padded_row[CPU_COL_MAP.mem_channels[1].is_read] = F::one();
                padded_row[CPU_COL_MAP.mem_channels[2].is_read] = F::zero();
            });
    }

    fn set_imm_value<F: Field>(&self, cols: &mut CpuCols<F>, imm: Option<Word<u8>>) {
        if let Some(imm) = imm {
            cols.opcode_flags.is_imm_op = F::one();
            cols.mem_channels[1].value = imm.transform(F::from_canonical_u8);
        }
    }
}

pub trait MachineWithCpuChip<F: Field>: MachineWithMemoryChip<F> {
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
    Imm32Instruction,
    ReadAdviceInstruction,
    StopInstruction
);

/// Non-deterministic instructions

impl<M, F> Instruction<M, F> for ReadAdviceInstruction
where
    M: MachineWithCpuChip<F>,
    F: Field,
{
    const OPCODE: u32 = READ_ADVICE;

    fn execute(_state: &mut M, _ops: Operands<i32>) {
        panic!("execute_with_advice should be called instead");
    }

    fn execute_with_advice<Adv>(state: &mut M, ops: Operands<i32>, advice: &mut Adv)
    where
        M: MachineWithCpuChip<F>,
        Adv: AdviceProvider,
    {
        let clk = state.cpu().clock;
        let fp = state.cpu().fp as i32;
        let mem_addr = fp + ops.a();

        // Read from the advice tape into memory
        let advice_byte = advice.get_advice().expect("No more advice");
        state
            .mem_mut()
            .write(clk, mem_addr as u32, Word::from_u8(advice_byte), true);

        state.cpu_mut().pc += 1;
        state.cpu_mut().push_op(
            Operation::ReadAdvice,
            <Self as Instruction<M, F>>::OPCODE,
            ops,
        );
    }
}

/// Deterministic instructions

impl<M, F> Instruction<M, F> for Load32Instruction
where
    M: MachineWithCpuChip<F>,
    F: Field,
{
    const OPCODE: u32 = LOAD32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let fp = state.cpu().fp;
        let read_addr_1 = (fp as i32 + ops.c()) as u32;
        let read_addr_2 = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let cell = state.mem_mut().read(
            clk,
            read_addr_2.into(),
            true,
            pc,
            opcode,
            1,
            &format!(
                "fp = {}, c = {}, [fp+c] = {:?}",
                fp as i32,
                ops.c() as u32,
                read_addr_2
            ),
        );
        state.mem_mut().write(clk, write_addr, cell, true);
        state.cpu_mut().pc += 1;
        state.cpu_mut().push_op(Operation::Load32, opcode, ops);
    }
}

impl<M, F> Instruction<M, F> for Store32Instruction
where
    M: MachineWithCpuChip<F>,
    F: Field,
{
    const OPCODE: u32 = STORE32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let read_addr = (state.cpu().fp as i32 + ops.c()) as u32;
        let write_addr_loc = (state.cpu().fp as i32 + ops.b()) as u32;
        let pc = state.cpu().pc;
        let write_addr = state
            .mem_mut()
            .read(clk, write_addr_loc.into(), true, pc, opcode, 0, "");
        let cell = state
            .mem_mut()
            .read(clk, read_addr, true, pc, opcode, 1, "");
        state.mem_mut().write(clk, write_addr.into(), cell, true);
        state.cpu_mut().pc += 1;
        state.cpu_mut().push_op(Operation::Store32, opcode, ops);
    }
}

impl<M, F> Instruction<M, F> for JalInstruction
where
    M: MachineWithCpuChip<F>,
    F: Field,
{
    const OPCODE: u32 = JAL;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        // Store 24 * (pc + 1) to local stack variable at offset a
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let next_pc = state.cpu().pc + 1;
        state
            .mem_mut()
            .write(clk, write_addr, (BYTES_PER_INSTR * next_pc).into(), true);
        // Set pc to the field element b / 24
        state.cpu_mut().pc = (ops.b() as u32) / BYTES_PER_INSTR;
        // Set fp to fp + c
        state.cpu_mut().fp = (state.cpu().fp as i32 + ops.c()) as u32;
        state
            .cpu_mut()
            .push_op(Operation::Jal, <Self as Instruction<M, F>>::OPCODE, ops);
    }
}

impl<M, F> Instruction<M, F> for JalvInstruction
where
    M: MachineWithCpuChip<F>,
    F: Field,
{
    const OPCODE: u32 = JALV;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        // Store pc + 1 to local stack variable at offset a
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let next_pc = state.cpu().pc + 1;
        state
            .mem_mut()
            .write(clk, write_addr, (BYTES_PER_INSTR * next_pc).into(), true);
        // Set pc to the field element [b]
        let read_addr = (state.cpu().fp as i32 + ops.b()) as u32;
        state.cpu_mut().pc = <Word<u8> as Into<u32>>::into(
            state
                .mem_mut()
                .read(clk, read_addr, true, pc, opcode, 0, ""),
        ) / BYTES_PER_INSTR;
        // Set fp to [c]
        let read_addr = (state.cpu().fp as i32 + ops.c()) as u32;
        let cell: u32 = state
            .mem_mut()
            .read(clk, read_addr, true, pc, opcode, 2, "")
            .into();
        state.cpu_mut().fp += cell;
        state.cpu_mut().push_op(Operation::Jalv, opcode, ops);
    }
}

impl<M, F> Instruction<M, F> for BeqInstruction
where
    M: MachineWithCpuChip<F>,
    F: Field,
{
    const OPCODE: u32 = BEQ;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let pc = state.cpu().pc;
        let cell_1 = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let cell_2 = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };
        if cell_1 == cell_2 {
            state.cpu_mut().pc = (ops.a() as u32) / BYTES_PER_INSTR;
        } else {
            state.cpu_mut().pc = state.cpu().pc + 1;
        }
        state.cpu_mut().push_op(Operation::Beq(imm), opcode, ops);
    }
}

impl<M, F> Instruction<M, F> for BneInstruction
where
    M: MachineWithCpuChip<F>,
    F: Field,
{
    const OPCODE: u32 = BNE;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let pc = state.cpu().pc;
        let cell_1 = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let cell_2 = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };
        if cell_1 != cell_2 {
            state.cpu_mut().pc = (ops.a() as u32) / BYTES_PER_INSTR;
        } else {
            state.cpu_mut().pc = state.cpu().pc + 1;
        }
        state.cpu_mut().push_op(Operation::Bne(imm), opcode, ops);
    }
}

impl<M, F> Instruction<M, F> for Imm32Instruction
where
    M: MachineWithCpuChip<F>,
    F: Field,
{
    const OPCODE: u32 = IMM32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let value = Word([ops.b() as u8, ops.c() as u8, ops.d() as u8, ops.e() as u8]);
        state.mem_mut().write(clk, write_addr, value.into(), true);
        state.cpu_mut().pc += 1;
        state
            .cpu_mut()
            .push_op(Operation::Imm32, <Self as Instruction<M, F>>::OPCODE, ops);
    }
}

impl<M, F> Instruction<M, F> for StopInstruction
where
    M: MachineWithCpuChip<F>,
    F: Field,
{
    const OPCODE: u32 = STOP;

    fn execute(state: &mut M, ops: Operands<i32>) {
        state.cpu_mut().pc = state.cpu().pc;
        state
            .cpu_mut()
            .push_op(Operation::Stop, <Self as Instruction<M, F>>::OPCODE, ops);
    }
}

impl CpuChip {
    pub fn push_bus_op_with_memory(
        &mut self,
        imm: Option<Word<u8>>,
        opcode: u32,
        operands: Operands<i32>,
    ) {
        self.pc += 1;
        self.push_op(Operation::BusWithMemory(imm), opcode, operands);
    }

    pub fn push_bus_op(&mut self, imm: Option<Word<u8>>, opcode: u32, operands: Operands<i32>) {
        self.pc += 1;
        self.push_op(Operation::Bus(imm), opcode, operands);
    }

    pub fn push_op(&mut self, op: Operation, opcode: u32, operands: Operands<i32>) {
        self.operations.push(op);
        self.instructions.push(InstructionWord { opcode, operands });
        self.save_register_state();
        self.clock += 1;
    }

    pub fn save_register_state(&mut self) {
        let registers = Registers {
            pc: self.pc,
            fp: self.fp,
        };
        self.registers.push(registers);
    }
}
