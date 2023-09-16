extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{Bitwise32Cols, COL_MAP, NUM_COLS};
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word};
use valida_opcodes::{AND32, OR32, XOR32};

use p3_air::VirtualPairCol;
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;
use valida_util::pad_to_power_of_two;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    And32(Word<u8>, Word<u8>, Word<u8>), // (dst, src1, src2)
    Or32(Word<u8>, Word<u8>, Word<u8>),  // ''
    Xor32(Word<u8>, Word<u8>, Word<u8>), // ''
}

#[derive(Default)]
pub struct Bitwise32Chip {
    pub operations: Vec<Operation>,
}

impl<F, M> Chip<M> for Bitwise32Chip
where
    F: PrimeField,
    M: MachineWithGeneralBus<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        let mut rows = self
            .operations
            .par_iter()
            .map(|op| self.op_to_row(op))
            .collect::<Vec<_>>();

        let mut trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS);

        pad_to_power_of_two::<NUM_COLS, F>(&mut trace.values);

        trace
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::new_main(
            vec![
                (COL_MAP.is_and, M::F::from_canonical_u32(AND32)),
                (COL_MAP.is_or, M::F::from_canonical_u32(OR32)),
                (COL_MAP.is_xor, M::F::from_canonical_u32(XOR32)),
            ],
            M::F::ZERO,
        );

        let input_1 = COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let is_real = VirtualPairCol::sum_main(vec![COL_MAP.is_and, COL_MAP.is_or, COL_MAP.is_xor]);

        let receive = Interaction {
            fields,
            count: is_real,
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }
}

impl Bitwise32Chip {
    fn op_to_row<F>(&self, op: &Operation) -> [F; NUM_COLS]
    where
        F: PrimeField,
    {
        let mut row = [F::ZERO; NUM_COLS];
        let cols: &mut Bitwise32Cols<F> = unsafe { transmute(&mut row) };

        match op {
            Operation::Xor32(a, b, c) => {
                cols.is_xor = F::ONE;
                self.set_cols(a, b, c, cols);
            }
            Operation::And32(a, b, c) => {
                cols.is_and = F::ONE;
                self.set_cols(a, b, c, cols);
            }
            Operation::Or32(a, b, c) => {
                cols.is_or = F::ONE;
                self.set_cols(a, b, c, cols);
            }
        }

        row
    }

    fn set_cols<F>(&self, a: &Word<u8>, b: &Word<u8>, c: &Word<u8>, cols: &mut Bitwise32Cols<F>)
    where
        F: PrimeField,
    {
        cols.input_1 = b.transform(F::from_canonical_u8);
        cols.input_2 = c.transform(F::from_canonical_u8);
        cols.output = a.transform(F::from_canonical_u8);
    }
}

pub trait MachineWithBitwise32Chip: MachineWithCpuChip {
    fn bitwise_u32(&self) -> &Bitwise32Chip;
    fn bitwise_u32_mut(&mut self) -> &mut Bitwise32Chip;
}

instructions!(And32Instruction, Or32Instruction, Xor32Instruction);

impl<M> Instruction<M> for Xor32Instruction
where
    M: MachineWithBitwise32Chip,
{
    const OPCODE: u32 = XOR32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;

        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true)
        };

        let a = b ^ c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .bitwise_u32_mut()
            .operations
            .push(Operation::Xor32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);
    }
}

impl<M> Instruction<M> for And32Instruction
where
    M: MachineWithBitwise32Chip,
{
    const OPCODE: u32 = AND32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true)
        };

        let a = b & c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .bitwise_u32_mut()
            .operations
            .push(Operation::And32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);
    }
}

impl<M> Instruction<M> for Or32Instruction
where
    M: MachineWithBitwise32Chip,
{
    const OPCODE: u32 = OR32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true)
        };

        let a = b | c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .bitwise_u32_mut()
            .operations
            .push(Operation::And32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);
    }
}
