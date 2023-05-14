#![no_std]

extern crate alloc;

use crate::columns::{MemoryCols, MEM_COL_MAP, MEM_LOOKUPS, NUM_MEM_COLS, NUM_MEM_LOOKUPS};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::mem::transmute;
use p3_field::{AbstractField, AsInt, Field};
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31 as Fp;
use valida_machine::{lookup::LogUp, Chip, Machine, Word, LOOKUP_DEGREE_BOUND};

pub mod columns;
mod stark;

#[derive(Copy, Clone)]
pub enum Operation {
    Read(Fp, Word<Fp>),
    Write(Fp, Word<Fp>),
    DummyRead(Fp, Word<Fp>),
}

impl Operation {
    pub fn get_address(&self) -> Fp {
        match self {
            Operation::Read(addr, _) => *addr,
            Operation::Write(addr, _) => *addr,
            Operation::DummyRead(addr, _) => *addr,
        }
    }
    pub fn get_value(&self) -> Word<Fp> {
        match self {
            Operation::Read(_, value) => *value,
            Operation::Write(_, value) => *value,
            Operation::DummyRead(_, value) => *value,
        }
    }
}

pub struct MemoryChip {
    pub cells: BTreeMap<Fp, Word<Fp>>,
    pub operations: BTreeMap<Fp, Vec<Operation>>,
}

pub trait MachineWithMemoryChip: Machine {
    fn mem(&self) -> &MemoryChip;
    fn mem_mut(&mut self) -> &mut MemoryChip;
}

impl MemoryChip {
    pub fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
            operations: BTreeMap::new(),
        }
    }

    pub fn read<A: Into<Fp> + Copy>(&mut self, clk: Fp, address: A, log: bool) -> Word<Fp> {
        let value = self.cells.get(&address.into()).copied().unwrap();
        if log {
            self.operations
                .entry(clk)
                .or_insert_with(Vec::new)
                .push(Operation::Read(address.into(), value));
        }
        value
    }

    pub fn write<V: Into<Word<Fp>> + Copy>(&mut self, clk: Fp, address: Fp, value: V, log: bool) {
        if log {
            self.operations
                .entry(clk)
                .or_insert_with(Vec::new)
                .push(Operation::Write(address, value.into()));
        }
        self.cells.insert(address, value.into());
    }
}

impl<M> Chip<M> for MemoryChip
where
    M: MachineWithMemoryChip,
{
    type F = Fp;
    type FE = Fp; // FIXME

    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<Self::F> {
        let mut ops = self
            .operations
            .iter()
            .flat_map(|(clk, ops)| {
                ops.iter()
                    .map(|op| (*clk, *op))
                    .collect::<Vec<(Fp, Operation)>>()
            })
            .collect::<Vec<_>>();

        // Sort first by addr, then by clk
        ops.sort_by_key(|(clk, op)| (op.get_address(), *clk));

        // Ensure consecutive sorted clock cycles for an address differ no more than
        // the length of the table
        Self::insert_dummy_reads(&mut ops);

        let mut rows = ops
            .into_iter()
            .enumerate()
            .map(|(n, (clk, op))| self.op_to_row(n, clk.as_canonical_uint() as usize, op, machine))
            .collect::<Vec<_>>();

        // Compute address difference values
        Self::compute_address_diffs(&mut rows);

        RowMajorMatrix::new(rows.concat(), NUM_MEM_COLS)
    }

    fn generate_permutation_trace(
        &self,
        machine: &M,
        main_trace: RowMajorMatrix<Self::F>,
        random_elements: Vec<Self::FE>,
    ) -> RowMajorMatrix<Self::F> {
        LogUp::<NUM_MEM_LOOKUPS, LOOKUP_DEGREE_BOUND>::new(MEM_LOOKUPS)
            .build_trace(&main_trace, random_elements)
    }
}

impl MemoryChip {
    fn op_to_row<N, M>(&self, n: N, clk: N, op: Operation, _machine: &M) -> [Fp; NUM_MEM_COLS]
    where
        N: Into<usize>,
        M: MachineWithMemoryChip,
    {
        let mut cols = MemoryCols::<Fp>::default();
        cols.clk = Fp::from(clk.into() as u32);
        cols.counter = Fp::from(n.into() as u32);

        match op {
            Operation::Read(addr, value) => {
                cols.is_read = Fp::ONE;
                cols.addr = addr;
                cols.value = value;
            }
            Operation::Write(addr, value) => {
                cols.addr = addr;
                cols.value = value;
            }
            Operation::DummyRead(addr, value) => {
                cols.addr = addr;
                cols.value = value;
                cols.is_dummy = Fp::ONE;
            }
        }

        let row: [Fp; NUM_MEM_COLS] = unsafe { transmute(cols) };
        row
    }

    fn insert_dummy_reads(ops: &mut Vec<(Fp, Operation)>) {
        let table_len = ops.len() as u32;
        let mut dummy_ops = Vec::new();
        for (op1, op2) in ops.iter().zip(ops.iter().skip(1)) {
            let addr_diff = op2.1.get_address() - op1.1.get_address();
            if addr_diff != Fp::ZERO {
                continue;
            }
            let clk_diff = (op2.0 - op1.0).as_canonical_uint();
            if clk_diff > table_len {
                let num_dummy_ops = clk_diff / table_len;
                for j in 0..num_dummy_ops {
                    let dummy_op_clk = op1.0 + Fp::from(table_len) * Fp::from(j as u32 + 1);
                    let dummy_op_addr = op1.1.get_address();
                    let dummy_op_value = op1.1.get_value();
                    dummy_ops.push((
                        dummy_op_clk,
                        Operation::DummyRead(dummy_op_addr, dummy_op_value),
                    ));
                }
            }
        }
        // TODO: Track number of operations at a given address instead of recounting here
        for (clk, op) in dummy_ops.iter() {
            let idx_addr = ops
                .binary_search_by_key(&op.get_address(), |(_, op)| op.get_address())
                .unwrap();
            let num_ops = ops[idx_addr..]
                .iter()
                .take_while(|(_, op2)| op.get_address() == op2.get_address())
                .count();
            let idx_clk =
                ops[idx_addr..(idx_addr + num_ops)].partition_point(|(clk2, _)| clk2 < clk);
            ops.insert(idx_addr + idx_clk, (*clk, *op));
        }
    }

    fn compute_address_diffs(rows: &mut Vec<[Fp; NUM_MEM_COLS]>) {
        // TODO: Use batch inversion
        for n in 0..(rows.len() - 1) {
            let addr = rows[n][MEM_COL_MAP.addr];
            let addr_next = rows[n][MEM_COL_MAP.addr];
            rows[n][MEM_COL_MAP.diff] = addr_next - addr;
            if (addr - addr_next) != Fp::ZERO {
                rows[n][MEM_COL_MAP.diff_inv] = (addr_next - addr).try_inverse().unwrap();
                rows[n][MEM_COL_MAP.addr_not_equal] = Fp::ONE;
            }
        }
    }
}
