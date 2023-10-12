extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use columns::{Mul32Cols, MUL_COL_MAP, NUM_MUL_COLS};
use core::iter::Sum;
use core::ops::Mul;
use itertools::iproduct;
use valida_bus::MachineWithGeneralBus;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, BusArgument, Chip, Instruction, Interaction, Operands, Word};
use valida_opcodes::MUL32;
use valida_range::MachineWithRangeChip;

use core::borrow::BorrowMut;
use p3_air::VirtualPairCol;
use p3_field::{PrimeField, PrimeField64};
use p3_matrix::dense::RowMajorMatrix;

pub mod columns;
pub mod stark;

#[derive(Clone)]
pub enum Operation {
    Mul32(Word<u8>, Word<u8>, Word<u8>),
}

#[derive(Default)]
pub struct Mul32Chip {
    pub operations: Vec<Operation>,
}

impl<F, M> Chip<M> for Mul32Chip
where
    F: PrimeField64,
    M: MachineWithGeneralBus<F = F>,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<M::F> {
        const MIN_LENGTH: usize = 1 << 10; // for the range check counter

        let num_ops = self.operations.len();
        let num_padded_ops = num_ops.next_power_of_two().max(MIN_LENGTH);
        let mut values = vec![F::ZERO; num_padded_ops * NUM_MUL_COLS];

        // Encode the real operations.
        for (i, op) in self.operations.iter().enumerate() {
            let row = &mut values[i * NUM_MUL_COLS..(i + 1) * NUM_MUL_COLS];
            let cols: &mut Mul32Cols<F> = row.borrow_mut();
            cols.counter = F::from_canonical_usize(i + 1);
            cols.is_real = F::ONE;
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

        // Set counter multiplicity
        let num_rows = values.len() / NUM_MUL_COLS;
        let mut mult = vec![F::ZERO; num_rows];
        for i in 0..num_rows {
            let r = values[MUL_COL_MAP.r + i * NUM_MUL_COLS].as_canonical_u64();
            let s = values[MUL_COL_MAP.s + i * NUM_MUL_COLS].as_canonical_u64();
            mult[r as usize] += F::ONE;
            mult[s as usize] += F::ONE;
        }

        RowMajorMatrix {
            values,
            width: NUM_MUL_COLS,
        }
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        let opcode = VirtualPairCol::constant(M::F::from_canonical_u32(MUL32));
        let input_1 = MUL_COL_MAP.input_1.0.map(VirtualPairCol::single_main);
        let input_2 = MUL_COL_MAP.input_2.0.map(VirtualPairCol::single_main);
        let output = MUL_COL_MAP.output.0.map(VirtualPairCol::single_main);

        let mut fields = vec![opcode];
        fields.extend(input_1);
        fields.extend(input_2);
        fields.extend(output);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(MUL_COL_MAP.is_real),
            argument_index: machine.general_bus(),
        };
        vec![receive]
    }

    fn local_sends(&self) -> Vec<Interaction<M::F>> {
        let send_r = Interaction {
            fields: vec![VirtualPairCol::single_main(MUL_COL_MAP.r)],
            count: VirtualPairCol::one(),
            argument_index: BusArgument::Local(0),
        };
        let send_s = Interaction {
            fields: vec![VirtualPairCol::single_main(MUL_COL_MAP.s)],
            count: VirtualPairCol::one(),
            argument_index: BusArgument::Local(0),
        };
        vec![send_r, send_s]
    }

    fn local_receives(&self) -> Vec<Interaction<M::F>> {
        let receives = Interaction {
            fields: vec![VirtualPairCol::single_main(MUL_COL_MAP.counter)],
            count: VirtualPairCol::single_main(MUL_COL_MAP.counter_mult),
            argument_index: BusArgument::Local(0),
        };
        vec![receives]
    }
}

impl Mul32Chip {
    fn op_to_row<F>(&self, op: &Operation, cols: &mut Mul32Cols<F>)
    where
        F: PrimeField,
    {
        match op {
            Operation::Mul32(a, b, c) => {
                cols.input_1 = b.transform(F::from_canonical_u8);
                cols.input_2 = c.transform(F::from_canonical_u8);
                cols.output = a.transform(F::from_canonical_u8);

                // Compute $r$ to satisfy $pi - z = 2^32 r$.
                let base_m32: [u64; 4] = [1, 1 << 8, 1 << 16, 1 << 24];
                let pi = pi_m::<4, u64, u64>(
                    &base_m32,
                    b.transform(|x| x as u64),
                    c.transform(|x| x as u64),
                );
                let z: u32 = (*a).into();
                let z: u64 = z as u64;
                let r = (pi - z) / (1u64 << 32);
                let r = r as u32;
                cols.r = F::from_canonical_u32(r);

                // Compute $s$ to satisfy $pi' - z' = 2^16 s$.
                let base_m16: [u32; 2] = [1, 1 << 8];
                let pi_prime = pi_m::<2, u32, u32>(
                    &base_m16,
                    b.transform(|x| x as u32),
                    c.transform(|x| x as u32),
                );
                let z_prime = a[3] as u32 + (1u32 << 8) * a[2] as u32;
                let z_prime: u32 = z_prime.into();
                let s = (pi_prime - z_prime) / (1u32 << 16);
                cols.s = F::from_canonical_u32(s);
            }
        }
    }
}

fn pi_m<const N: usize, I: Copy, O: Mul<I, Output = O> + Clone + Sum>(
    base: &[O; N],
    input_1: Word<I>,
    input_2: Word<I>,
) -> O {
    iproduct!(0..N, 0..N)
        .filter(|(i, j)| i + j < N)
        .map(|(i, j)| base[i + j].clone() * input_1[3 - i] * input_2[3 - j])
        .sum()
}

fn sigma_m<const N: usize, I, O: Mul<I, Output = O> + Clone + Sum>(
    base: &[O],
    input: Word<I>,
) -> O {
    input
        .into_iter()
        .rev()
        .take(N)
        .enumerate()
        .map(|(i, x)| base[i].clone() * x)
        .sum()
}

pub trait MachineWithMul32Chip: MachineWithCpuChip {
    fn mul_u32(&self) -> &Mul32Chip;
    fn mul_u32_mut(&mut self) -> &mut Mul32Chip;
}

instructions!(Mul32Instruction);

impl<M> Instruction<M> for Mul32Instruction
where
    M: MachineWithMul32Chip + MachineWithRangeChip<256>,
{
    const OPCODE: u32 = MUL32;

    fn execute(state: &mut M, ops: Operands<i32>) {
        let clk = state.cpu().clock;
        let mut imm: Option<Word<u8>> = None;
        let read_addr_1 = (state.cpu().fp as i32 + ops.b()) as u32;
        let write_addr = (state.cpu().fp as i32 + ops.a()) as u32;
        let b = state.mem_mut().read(clk, read_addr_1, true);
        let c: Word<u8> = if ops.is_imm() == 1 {
            let c = (ops.c() as u32).into();
            imm = Some(c);
            c
        } else {
            let read_addr_2 = (state.cpu().fp as i32 + ops.c()) as u32;
            state.mem_mut().read(clk, read_addr_2, true).into()
        };

        let a = b * c;
        state.mem_mut().write(clk, write_addr, a, true);

        state
            .mul_u32_mut()
            .operations
            .push(Operation::Mul32(a, b, c));
        state
            .cpu_mut()
            .push_bus_op(imm, <Self as Instruction<M>>::OPCODE, ops);

        state.range_check(a);
    }
}
