extern crate alloc;

use alloc::vec::Vec;
use columns::NUM_MUL_COLS;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{instructions, Instruction, Operands, Word, MEMORY_CELL_BYTES};

use p3_field::{AbstractField, PrimeField, PrimeField32};
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31 as Fp;
use valida_machine::Chip;

pub mod columns;
mod stark;

#[derive(Clone)]
pub enum Operation {
    Mul32,
}

#[derive(Default)]
pub struct Mul32Chip<F> {
    pub clock: F,
    pub operations: Vec<Operation>,
}

impl<M> Chip<M> for Mul32Chip<Fp>
where
    M: MachineWithMul32Chip<F = Fp>,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F> {
        let rows = self
            .operations
            .iter()
            .cloned()
            .enumerate()
            .map(|(n, op)| self.op_to_row(op, machine))
            .collect::<Vec<_>>();
        RowMajorMatrix::new(rows.concat(), NUM_MUL_COLS)
    }
}

impl<F> Mul32Chip<F> {
    fn op_to_row<M>(&self, op: Operation, _machine: &M) -> [Fp; NUM_MUL_COLS]
    where
        M: MachineWithMul32Chip,
    {
        todo!()
    }
}

pub trait MachineWithMul32Chip: MachineWithCpuChip {
    fn mul_u32(&self) -> &Mul32Chip<Self::F>;
    fn mul_u32_mut(&mut self) -> &mut Mul32Chip<Self::F>;
}

instructions!(Mul32Instruction);

impl<M: MachineWithMul32Chip<F = Fp>> Instruction<M> for Mul32Instruction {
    const OPCODE: u32 = 10;

    fn execute(state: &mut M, ops: Operands<Fp>) {
        let clk = state.cpu().clock;
        let read_addr_1 = state.cpu().fp + ops.b();
        let write_addr = state.cpu().fp + ops.a();
        let b: [u32; 4] = state.mem_mut().read(clk, read_addr_1, true).into();
        let c: [u32; 4] = if ops.is_imm() == Fp::ONE {
            let bytes = ops.c().as_canonical_u32().to_be_bytes();
            bytes
                .chunks_exact(MEMORY_CELL_BYTES)
                .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        } else {
            let read_addr_2 = state.cpu().fp + ops.c();
            state.mem_mut().read(clk, read_addr_2, true).into()
        };

        let res = b[3] * c[3]
            + ((b[3] * c[2] + b[2] * c[3]) << 8)
            + ((b[3] * c[1] + b[2] * c[2] + b[1] * c[3]) << 16)
            + ((b[3] * c[0] + b[2] * c[1] + b[1] * c[2] + b[0] * c[3]) << 24);
        let mut a = Word::<Fp>::default();
        a[0] = Fp::from_canonical_u32(res & 0xff);
        a[1] = Fp::from_canonical_u32((res >> 8) & 0xff);
        a[2] = Fp::from_canonical_u32((res >> 16) & 0xff);
        a[3] = Fp::from_canonical_u32((res >> 24) & 0xff);
        state.mem_mut().write(clk, write_addr, a, true);

        state.mul_u32_mut().operations.push(Operation::Mul32);
        state.cpu_mut().clock += Fp::ONE;
        state.cpu_mut().pc += Fp::ONE;
    }
}
