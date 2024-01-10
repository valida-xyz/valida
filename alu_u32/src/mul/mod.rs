extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{Mul32Cols, MUL_COL_MAP, NUM_MUL_COLS};
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Chip, Instruction, Interaction, Operands, Word, Mulhs, Mulhu};
use valida_opcodes::{MUL32, MULHS32, MULHU32};
use valida_range::MachineWithRangeChip;

use core::borrow::BorrowMut;
use p3_air::VirtualPairCol;
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Mul32(Word<u8>, Word<u8>, Word<u8>),
    Mulhs32(Word<u8>, Word<u8>, Word<u8>),
    Mulhu32(Word<u8>, Word<u8>, Word<u8>),
}

#[derive(Default)]
pub struct Mul32Chip {
    pub operations: Vec<Operation>,
}

impl<F, M> Chip<M> for Mul32Chip
where
    F: PrimeField,
    M: MachineWithGeneralBus<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        const MIN_LENGTH: usize = 1 << 10; // for the range check counter

        let num_ops = self.operations.len();
        let num_padded_ops = num_ops.next_power_of_two().max(MIN_LENGTH);
        let mut values = vec![F::zero(); num_padded_ops * NUM_MUL_COLS];

        // Encode the real operations.
        for (i, op) in self.operations.iter().enumerate() {
            let row = &mut values[i * NUM_MUL_COLS..(i + 1) * NUM_MUL_COLS];
            let cols: &mut Mul32Cols<F> = row.borrow_mut();
            cols.counter = F::from_canonical_usize(i + 1);
            self.op_to_row(op, cols);
        }

        // Encode dummy operations as needed to pad the trace.
        let dummy_op = Operation::Mul32(Word::default(), Word::default(), Word::default());
        for i in num_ops..num_padded_ops {
            let row = &mut values[i * NUM_MUL_COLS..(i + 1) * NUM_MUL_COLS];
            let cols: &mut Mul32Cols<F> = row.borrow_mut();
            cols.counter = F::from_canonical_usize(i + 1);
            self.op_to_row(&dummy_op, cols);
        }

        RowMajorMatrix {
            values,
            width: NUM_MUL_COLS,
        }
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::new_main(
            vec![
                (MUL_COL_MAP.is_mul, M::F::from_canonical_u32(MUL32)),
                (MUL_COL_MAP.is_mulhs, M::F::from_canonical_u32(MULHS32)),
                (MUL_COL_MAP.is_mulhu, M::F::from_canonical_u32(MULHU32)),
            ],
            M::F::zero(),
        );
        let input_1 = MUL_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = MUL_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = MUL_COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let is_real = 
            VirtualPairCol::sum_main(vec![MUL_COL_MAP.is_mul, MUL_COL_MAP.is_mulhs, MUL_COL_MAP.is_mulhu]);

        let receive = Interaction {
            fields,
            count: is_real,
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }

    fn local_sends(&self) -> Vec<Interaction<M::F>> {
        // TODO
        vec![]
    }
}

impl Mul32Chip {
    fn op_to_row<F>(&self, op: &Operation, cols: &mut Mul32Cols<F>)
    where
        F: PrimeField,
    {
        match op {
            Operation::Mul32(a, b, c) => {
                cols.is_mul = F::one();
                self.set_cols(a, b, c, cols);
            }
            Operation::Mulhs32(a, b, c) => {
                cols.is_mulhs = F::one();
                self.set_cols(a, b, c, cols);
            }
            Operation::Mulhu32(a, b, c) => {
                cols.is_mulhu = F::one();
                self.set_cols(a, b, c, cols);
            }
        }
    }
    
        fn set_cols<F>(&self, a: &Word<u8>, b: &Word<u8>, c: &Word<u8>, cols: &mut Mul32Cols<F>)
    where
        F: PrimeField,
    {
        cols.input_1 = b.transform(F::from_canonical_u8);
        cols.input_2 = c.transform(F::from_canonical_u8);
        cols.output = a.transform(F::from_canonical_u8);
    }
}

pub trait MachineWithMul32Chip: MachineWithCpuChip {
    fn mul_u32(&self) -> &Mul32Chip;
    fn mul_u32_mut(&mut self) -> &mut Mul32Chip;
}

instructions!(Mul32Instruction, Mulhs32Instruction, Mulhu32Instruction);

impl<M> Instruction<M> for Mul32Instruction
where
    M: MachineWithMul32Chip + MachineWithRangeChip<256>,
{
    const OPCODE: u32 = MUL32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c: Word<u8> = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true, pc, opcode, 1, "").into()
        };

        let a = b * c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .mul_u32_mut()
            .operations
            .push(Operation::Mul32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, opcode, ops);

        state.range_check(a);
    }
}

impl<M> Instruction<M> for Mulhs32Instruction
where
    M: MachineWithMul32Chip + MachineWithRangeChip<256>,
{
    const OPCODE: u32 = MULHS32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c: Word<u8> = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true, pc, opcode, 1, "").into()
        };

        let a = b.mulhs(c);
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .mul_u32_mut()
            .operations
            .push(Operation::Mulhs32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, opcode, ops);

        state.range_check(a);
    }
}

impl<M> Instruction<M> for Mulhu32Instruction
where
    M: MachineWithMul32Chip + MachineWithRangeChip<256>,
{
    const OPCODE: u32 = MULHU32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let opcode = <Self as Instruction<M>>::OPCODE;
        let clk = state.cpu().clock;
        let pc = state.cpu().pc;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true, pc, opcode, 0, "");
        let c: Word<u8> = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true, pc, opcode, 1, "").into()
        };

        let a = b.mulhu(c);
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .mul_u32_mut()
            .operations
            .push(Operation::Mulhu32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, opcode, ops);

        state.range_check(a);
    }
}
