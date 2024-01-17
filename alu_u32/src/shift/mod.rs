extern crate alloc;

use crate::div::{MachineWithDiv32Chip, Operation as DivOperation};
use crate::mul::{MachineWithMul32Chip, Operation as MulOperation};
use alloc::vec;
use alloc::vec::Vec;
use columns::{Shift32Cols, COL_MAP, NUM_SHIFT_COLS};
use core::mem::transmute;
use valida_bus::{MachineWithGeneralBus, MachineWithRangeBus8};
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Sra, Word};
use valida_opcodes::{DIV32, MUL32, SDIV32, SHL32, SHR32, SRA32};

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;
use valida_machine::config::StarkConfig;
use valida_util::pad_to_power_of_two;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Shl32(Word<u8>, Word<u8>, Word<u8>), // (dst, src, shift)
    Shr32(Word<u8>, Word<u8>, Word<u8>), // ''
    Sra32(Word<u8>, Word<u8>, Word<u8>), // ''
}

#[derive(Default)]
pub struct Shift32Chip {
    pub operations: Vec<Operation>,
}

impl<M, SC> Chip<M, SC> for Shift32Chip
where
    M: MachineWithGeneralBus<SC::Val> + MachineWithRangeBus8<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        let rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();

        let mut trace = RowMajorMatrix::new(
            rows.into_iter().flatten().collect::<Vec<_>>(),
            NUM_SHIFT_COLS,
        );

        pad_to_power_of_two::<NUM_SHIFT_COLS, SC::Val>(&mut trace.values);

        trace
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let opcode = VirtualPairCol::new_main(
            vec![
                (COL_MAP.is_shl, SC::Val::from_canonical_u32(MUL32)),
                (COL_MAP.is_shr, SC::Val::from_canonical_u32(DIV32)),
                (COL_MAP.is_sra, SC::Val::from_canonical_u32(SDIV32)),
            ],
            SC::Val::zero(),
        );
        let input_1 = COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = COL_MAP.power_of_two.0.map(VirtualPairCol::single_main);
        let output = COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let is_real =
            VirtualPairCol::sum_main(vec![COL_MAP.is_shl, COL_MAP.is_shr, COL_MAP.is_sra]);

        let send = Interaction {
            fields,
            count: is_real,
            argument_index: machine.general_bus(),
        };

        vec![send]
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let opcode = VirtualPairCol::new_main(
            vec![
                (COL_MAP.is_shl, SC::Val::from_canonical_u32(SHL32)),
                (COL_MAP.is_shr, SC::Val::from_canonical_u32(SHR32)),
                (COL_MAP.is_sra, SC::Val::from_canonical_u32(SRA32)),
            ],
            SC::Val::zero(),
        );
        let input_1 = COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let is_real =
            VirtualPairCol::sum_main(vec![COL_MAP.is_shl, COL_MAP.is_shr, COL_MAP.is_sra]);

        let receive = Interaction {
            fields,
            count: is_real,
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl Shift32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_SHIFT_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::zero(); NUM_SHIFT_COLS];
        let cols: &mut Shift32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Shr32(a, b, c) => {
                cols.is_shr = F::one();
                self.set_cols(cols, a, b, c);
            }
            Operation::Sra32(a, b, c) => {
                cols.is_sra = F::one();
                self.set_cols(cols, a, b, c);
            }
            Operation::Shl32(a, b, c) => {
                cols.is_shl = F::one();
                self.set_cols(cols, a, b, c);
            }
        }

        row
    }

    fn set_cols<F>(&self, cols: &mut Shift32Cols<F>, a: &Word<u8>, b: &Word<u8>, c: &Word<u8>)
    where
        F: PrimeField,
    {
        // Set the input columns
        cols.input_1 = b.transform(F::from_canonical_u8);
        cols.input_2 = c.transform(F::from_canonical_u8);
        cols.output = a.transform(F::from_canonical_u8);

        // Set individual bits columns (using least significant byte of input_2)
        for i in 0..8 {
            cols.bits_2[i] = F::from_canonical_u8(c[3] >> i & 1);
        }

        // Compute the temporary value: 2^{bits_2[0] + 2*bits_2[1] + 4*bits_2[2]}
        cols.temp_1 =
            F::from_canonical_u8((c[3] & 0b1) + 2 * ((c[3] >> 1) & 0b1) + 4 * ((c[3] >> 2) & 0b1));

        cols.power_of_two = (Word::from(1) << *c).transform(F::from_canonical_u8);
    }
}

pub trait MachineWithShift32Chip<F: Field>: MachineWithCpuChip<F> {
    fn shift_u32(&self) -> &Shift32Chip;
    fn shift_u32_mut(&mut self) -> &mut Shift32Chip;
}

instructions!(Shl32Instruction, Shr32Instruction, Sra32Instruction);

impl<M, F> Instruction<M, F> for Shl32Instruction
where
    M: MachineWithShift32Chip<F> + MachineWithMul32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = SHL32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        // Write the shifted value to memory
        let a = Word::from(b << c);
        state.mem_mut().write(clk, write_addr, a, true);

        // Add a "receive" multiplication operation to match the "send"
        let d = Word::from(1) << c;
        state
            .mul_u32_mut()
            .operations
            .push(MulOperation::Mul32(a, b, d));

        state
            .shift_u32_mut()
            .operations
            .push(Operation::Shl32(a, b, c));
        state.cpu_mut().push_bus_op(imm, opcode, ops);
    }
}

impl<M, F> Instruction<M, F> for Shr32Instruction
where
    M: MachineWithShift32Chip<F> + MachineWithDiv32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = SHR32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        // Write the shifted value to memory
        let a = Word::from(b >> c);
        state.mem_mut().write(clk, write_addr, a, true);

        // Add a "receive" division operation to match the "send"
        let d = Word::from(1) << c;
        state
            .div_u32_mut()
            .operations
            .push(DivOperation::Div32(a, b, d));

        state
            .shift_u32_mut()
            .operations
            .push(Operation::Shr32(a, b, c));
        state.cpu_mut().push_bus_op(imm, opcode, ops);
    }
}

impl<M, F> Instruction<M, F> for Sra32Instruction
where
    M: MachineWithShift32Chip<F> + MachineWithDiv32Chip<F>,
    F: Field,
{
    const OPCODE: u32 = SRA32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M, F>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state
            .mem_mut()
            .read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state
                .mem_mut()
                .read(clk, read_addr_2, true, pc, opcode, 1, "")
        };

        // Write the shifted value to memory
        let a = b.sra(c);
        state.mem_mut().write(clk, write_addr, a, true);

        // Add a "receive" division operation to match the "send"
        let d = Word::from(1) << c;
        state
            .div_u32_mut()
            .operations
            .push(DivOperation::SDiv32(a, b, d));

        state
            .shift_u32_mut()
            .operations
            .push(Operation::Shr32(a, b, c));
        state.cpu_mut().push_bus_op(imm, opcode, ops);
    }
}
