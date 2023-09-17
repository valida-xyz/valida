extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{Bitwise32Cols, COL_MAP, NUM_COLS};
use core::mem::transmute;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{
    instructions, BusArgument, Chip, Instruction, Interaction, Operands, Word, MEMORY_CELL_BYTES,
};
use valida_opcodes::{AND32, OR32, XOR32};

use p3_air::VirtualPairCol;
use p3_field::{AbstractField, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;

pub mod columns;
pub mod stark;

/// Commits the bitwise op (i1, i2, o1) with a base-256 encoding where i1, i2, o1 \in [0, 8).
/// Assumes the field is large enough to encode the result (~2^25).
#[inline]
pub fn commit_bitwise_op<F: AbstractField>(i1: F, i2: F, o1: F) -> F {
    let b1 = F::from_canonical_usize(1);
    let b2 = F::from_canonical_usize(1 << 8);
    let b3 = F::from_canonical_usize(1 << 16);
    i1 * b1 + i2 * b2 + o1 * b3
}

/// Calculates which row contains the multiplicity for the given input byts.
pub fn multiplicity_idx_from_bytes(i1: u8, i2: u8) -> usize {
    (i1 as usize) + (i2 as usize) * 256
}

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

        // Calculate trace length after padding.
        let bitwise_ops_table_size = 1 << 16;
        let nb_rows = if rows.len() < bitwise_ops_table_size {
            bitwise_ops_table_size
        } else {
            rows.len().next_power_of_two()
        };

        // Mutliplicity vectors to count how many times a certain bitwise operation among
        // bytes was performed.
        let mut byte_mult = vec![F::ZERO; 8];
        let mut and_mult = vec![F::ZERO; bitwise_ops_table_size];
        let mut or_mult = vec![F::ZERO; bitwise_ops_table_size];
        let mut xor_mult = vec![F::ZERO; bitwise_ops_table_size];

        // Count the multiplicity of each bitwise operation.
        for op in self.operations.iter() {
            match op {
                Operation::And32(a, b, c) => {
                    for i in 0..MEMORY_CELL_BYTES {
                        let idx = multiplicity_idx_from_bytes(b.0[i], c.0[i]);
                        and_mult[idx] += F::ONE;
                        or_mult[0] += F::ONE;
                        xor_mult[0] += F::ONE;
                        byte_mult[a.0[i] as usize] += F::ONE;
                        byte_mult[b.0[i] as usize] += F::ONE;
                        byte_mult[c.0[i] as usize] += F::ONE;
                    }
                }
                Operation::Or32(a, b, c) => {
                    for i in 0..MEMORY_CELL_BYTES {
                        let idx = multiplicity_idx_from_bytes(b.0[i], c.0[i]);
                        and_mult[0] += F::ONE;
                        or_mult[idx] += F::ONE;
                        xor_mult[0] += F::ONE;
                        byte_mult[a.0[i] as usize] += F::ONE;
                        byte_mult[b.0[i] as usize] += F::ONE;
                        byte_mult[c.0[i] as usize] += F::ONE;
                    }
                }
                Operation::Xor32(a, b, c) => {
                    for i in 0..MEMORY_CELL_BYTES {
                        let idx = multiplicity_idx_from_bytes(b.0[i], c.0[i]);
                        and_mult[0] += F::ONE;
                        or_mult[0] += F::ONE;
                        xor_mult[idx] += F::ONE;
                        byte_mult[a.0[i] as usize] += F::ONE;
                        byte_mult[b.0[i] as usize] += F::ONE;
                        byte_mult[c.0[i] as usize] += F::ONE;
                    }
                }
            }
        }

        // Pad the trace with zeros and setup the lookup arguments.
        for i in 0..nb_rows {
            if i >= rows.len() {
                rows.push([F::ZERO; NUM_COLS]);
            }

            // Set the byte range lookup column.
            rows[i][COL_MAP.byte_lookup] = if i < 8 {
                F::from_canonical_usize(i)
            } else {
                F::from_canonical_usize(7)
            };
            rows[i][COL_MAP.byte_mult] = byte_mult[i];

            // Compute the input bytes to the bitwise operation.
            let i1: u8 = (i % 256) as u8;
            let i2: u8 = (i / 256) as u8;

            // Set the bitwise and column.
            let and = i1 & i2;
            let cand = commit_bitwise_op(
                F::from_canonical_u8(i1),
                F::from_canonical_u8(i2),
                F::from_canonical_u8(and),
            );
            rows[i][COL_MAP.and_lookup] = cand;
            rows[i][COL_MAP.and_mult] = and_mult[i];

            // Set the bitwise or column.
            let or = i1 | i2;
            let cor = commit_bitwise_op(
                F::from_canonical_u8(i1),
                F::from_canonical_u8(i2),
                F::from_canonical_u8(or),
            );
            rows[i][COL_MAP.or_lookup] = cor;
            rows[i][COL_MAP.or_mult] = or_mult[i];

            // Set the bitwise xor column.
            let xor = i1 ^ i2;
            let cxor = commit_bitwise_op(
                F::from_canonical_u8(i1),
                F::from_canonical_u8(i2),
                F::from_canonical_u8(xor),
            );
            rows[i][COL_MAP.xor_lookup] = cxor;
            rows[i][COL_MAP.xor_mult] = xor_mult[i];
        }

        let trace = RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_COLS);
        trace
    }

    fn local_sends(&self) -> Vec<Interaction<M::F>> {
        let byte = Interaction {
            fields: vec![
                VirtualPairCol::single_main(COL_MAP.input_1[0]),
                VirtualPairCol::single_main(COL_MAP.input_1[1]),
                VirtualPairCol::single_main(COL_MAP.input_1[2]),
                VirtualPairCol::single_main(COL_MAP.input_1[3]),
                VirtualPairCol::single_main(COL_MAP.input_2[0]),
                VirtualPairCol::single_main(COL_MAP.input_2[1]),
                VirtualPairCol::single_main(COL_MAP.input_2[2]),
                VirtualPairCol::single_main(COL_MAP.input_2[3]),
                VirtualPairCol::single_main(COL_MAP.output[0]),
                VirtualPairCol::single_main(COL_MAP.output[1]),
                VirtualPairCol::single_main(COL_MAP.output[2]),
                VirtualPairCol::single_main(COL_MAP.output[3]),
            ],
            count: VirtualPairCol::one(),
            argument_index: BusArgument::Local(0),
        };
        let and = Interaction {
            fields: vec![
                VirtualPairCol::single_main(COL_MAP.and[0]),
                VirtualPairCol::single_main(COL_MAP.and[1]),
                VirtualPairCol::single_main(COL_MAP.and[2]),
                VirtualPairCol::single_main(COL_MAP.and[3]),
            ],
            count: VirtualPairCol::one(),
            argument_index: BusArgument::Local(1),
        };
        let or = Interaction {
            fields: vec![
                VirtualPairCol::single_main(COL_MAP.or[0]),
                VirtualPairCol::single_main(COL_MAP.or[1]),
                VirtualPairCol::single_main(COL_MAP.or[2]),
                VirtualPairCol::single_main(COL_MAP.or[3]),
            ],
            count: VirtualPairCol::one(),
            argument_index: BusArgument::Local(2),
        };
        let xor = Interaction {
            fields: vec![
                VirtualPairCol::single_main(COL_MAP.xor[0]),
                VirtualPairCol::single_main(COL_MAP.xor[1]),
                VirtualPairCol::single_main(COL_MAP.xor[2]),
                VirtualPairCol::single_main(COL_MAP.xor[3]),
            ],
            count: VirtualPairCol::one(),
            argument_index: BusArgument::Local(3),
        };
        vec![byte, and, or, xor]
    }

    fn local_receives(&self) -> Vec<Interaction<M::F>> {
        let byte = Interaction {
            fields: vec![VirtualPairCol::single_main(COL_MAP.byte_lookup)],
            count: VirtualPairCol::single_main(COL_MAP.byte_mult),
            argument_index: BusArgument::Local(0),
        };
        let and = Interaction {
            fields: vec![VirtualPairCol::single_main(COL_MAP.and_lookup)],
            count: VirtualPairCol::single_main(COL_MAP.and_mult),
            argument_index: BusArgument::Local(1),
        };
        let or = Interaction {
            fields: vec![VirtualPairCol::single_main(COL_MAP.and_lookup)],
            count: VirtualPairCol::single_main(COL_MAP.and_mult),
            argument_index: BusArgument::Local(2),
        };
        let xor = Interaction {
            fields: vec![VirtualPairCol::single_main(COL_MAP.and_lookup)],
            count: VirtualPairCol::single_main(COL_MAP.and_mult),
            argument_index: BusArgument::Local(3),
        };
        vec![byte, and, or, xor]
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
        // Set inputs and outputs.
        cols.input_1 = b.transform(F::from_canonical_u8);
        cols.input_2 = c.transform(F::from_canonical_u8);
        cols.output = a.transform(F::from_canonical_u8);

        // Set up columns that get looked up to check the result.
        for i in 0..MEMORY_CELL_BYTES {
            cols.and[i] =
                cols.is_and * commit_bitwise_op(cols.input_1[i], cols.input_2[i], cols.output[i]);
            cols.or[i] =
                cols.is_or * commit_bitwise_op(cols.input_1[i], cols.input_2[i], cols.output[i]);
            cols.xor[i] =
                cols.is_xor * commit_bitwise_op(cols.input_1[i], cols.input_2[i], cols.output[i]);
        }
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
