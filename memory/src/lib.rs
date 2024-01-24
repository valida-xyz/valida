#![no_std]

extern crate alloc;

use crate::columns::{MemoryCols, MEM_COL_MAP, NUM_MEM_COLS};
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use core::mem::transmute;
use p3_air::VirtualPairCol;
use p3_field::{Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::*;
use valida_bus::MachineWithMemBus;
use valida_machine::StarkConfig;
use valida_machine::{BusArgument, Chip, Interaction, Machine, Word};
use valida_util::batch_multiplicative_inverse_allowing_zero;

pub mod columns;
pub mod stark;

#[derive(Copy, Clone, Debug)]
pub enum Operation {
    Read(u32, Word<u8>),
    Write(u32, Word<u8>),
    DummyRead(u32, Word<u8>),
}

impl Operation {
    pub fn get_address(&self) -> u32 {
        match self {
            Operation::Read(addr, _) => *addr,
            Operation::Write(addr, _) => *addr,
            Operation::DummyRead(addr, _) => *addr,
        }
    }
    pub fn get_value(&self) -> Word<u8> {
        match self {
            Operation::Read(_, value) => *value,
            Operation::Write(_, value) => *value,
            Operation::DummyRead(_, value) => *value,
        }
    }
}

#[derive(Default)]
pub struct MemoryChip {
    pub cells: BTreeMap<u32, Word<u8>>,
    pub operations: BTreeMap<u32, Vec<Operation>>,
}

pub trait MachineWithMemoryChip<F: Field>: Machine<F> {
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

    pub fn read(
        &mut self,
        clk: u32,
        address: u32,
        log: bool,
        pc: u32,
        opcode: u32,
        ordinal: u32,
        extra_info: &str,
    ) -> Word<u8> {
        let value = self.cells.get(&address.into()).copied()
          .unwrap_or_else(|| panic!("memory chip: read before write: {} (pc = {}, opcode = {}, ordinal = {}, extra_info = {})", address, pc, opcode, ordinal, extra_info));
        if log {
            self.operations
                .entry(clk)
                .or_insert_with(Vec::new)
                .push(Operation::Read(address.into(), value));
        }
        value
    }

    pub fn write(&mut self, clk: u32, address: u32, value: Word<u8>, log: bool) {
        if log {
            self.operations
                .entry(clk)
                .or_insert_with(Vec::new)
                .push(Operation::Write(address, value));
        }
        self.cells.insert(address, value.into());
    }
}

impl<M, SC> Chip<M, SC> for MemoryChip
where
    M: MachineWithMemBus<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
        let mut ops = self
            .operations
            .par_iter()
            .map(|(clk, ops)| {
                ops.iter()
                    .map(|op| (*clk, *op))
                    .collect::<Vec<(u32, Operation)>>()
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        // Sort first by addr, then by clk
        ops.sort_by_key(|(clk, op)| (op.get_address(), *clk));

        // Consecutive sorted clock cycles for an address should differ no more
        // than the length of the table (capped at 2^29)
        Self::insert_dummy_reads(&mut ops);

        let mut rows = ops
            .par_iter()
            .enumerate()
            .map(|(n, (clk, op))| self.op_to_row(n, *clk as usize, *op))
            .collect::<Vec<_>>();

        // Compute address difference values
        Self::compute_address_diffs(ops, &mut rows);

        let trace =
            RowMajorMatrix::new(rows.into_iter().flatten().collect::<Vec<_>>(), NUM_MEM_COLS);

        trace
    }

    fn local_sends(&self) -> Vec<Interaction<SC::Val>> {
        let sends = Interaction {
            fields: vec![VirtualPairCol::single_main(MEM_COL_MAP.diff)],
            count: VirtualPairCol::one(),
            argument_index: BusArgument::Local(0),
        };
        vec![sends]
    }

    fn local_receives(&self) -> Vec<Interaction<SC::Val>> {
        let receives = Interaction {
            fields: vec![VirtualPairCol::single_main(MEM_COL_MAP.counter)],
            count: VirtualPairCol::single_main(MEM_COL_MAP.counter_mult),
            argument_index: BusArgument::Local(0),
        };
        vec![receives]
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let is_read: VirtualPairCol<SC::Val> = VirtualPairCol::single_main(MEM_COL_MAP.is_read);
        let clk = VirtualPairCol::single_main(MEM_COL_MAP.clk);
        let addr = VirtualPairCol::single_main(MEM_COL_MAP.addr);
        let value = MEM_COL_MAP.value.0.map(VirtualPairCol::single_main);

        let mut fields = vec![is_read, clk, addr];
        fields.extend(value);

        let receive = Interaction {
            fields,
            count: VirtualPairCol::single_main(MEM_COL_MAP.is_real),
            argument_index: machine.mem_bus(),
        };
        vec![receive]
    }
}

impl MemoryChip {
    fn op_to_row<F: PrimeField>(&self, n: usize, clk: usize, op: Operation) -> [F; NUM_MEM_COLS] {
        let mut row = [F::zero(); NUM_MEM_COLS];
        let cols: &mut MemoryCols<F> = unsafe { transmute(&mut row) };

        cols.clk = F::from_canonical_usize(clk);
        cols.counter = F::from_canonical_usize(n);

        match op {
            Operation::Read(addr, value) => {
                cols.addr = F::from_canonical_u32(addr);
                cols.value = value.transform(F::from_canonical_u8);
                cols.is_read = F::one();
                cols.is_real = F::one();
            }
            Operation::Write(addr, value) => {
                cols.addr = F::from_canonical_u32(addr);
                cols.value = value.transform(F::from_canonical_u8);
                cols.is_real = F::one();
            }
            Operation::DummyRead(addr, value) => {
                cols.addr = F::from_canonical_u32(addr);
                cols.value = value.transform(F::from_canonical_u8);
                cols.is_read = F::one();
            }
        }

        row
    }

    fn insert_dummy_reads(ops: &mut Vec<(u32, Operation)>) {
        if ops.is_empty() {
            return;
        }

        let table_len = ops.len() as u32;
        let mut dummy_ops = Vec::new();
        for (op1, op2) in ops.iter().zip(ops.iter().skip(1)) {
            let addr_diff = op2.1.get_address() - op1.1.get_address();
            if addr_diff != 0 {
                // Add dummy reads when addr_diff is greater than the number of operations
                if addr_diff > table_len {
                    let num_dummy_ops = addr_diff / table_len;
                    for i in 0..num_dummy_ops {
                        let dummy_op_clk = op1.0;
                        let dummy_op_addr = op1.1.get_address() + table_len * (i + 1);
                        let dummy_op_value = op1.1.get_value();
                        dummy_ops.push((
                            dummy_op_clk,
                            Operation::DummyRead(dummy_op_addr, dummy_op_value),
                        ));
                    }
                } else {
                    continue;
                }
            } else {
                let clk_diff = op2.0 - op1.0;
                if clk_diff > table_len {
                    let num_dummy_ops = clk_diff / table_len;
                    for j in 0..num_dummy_ops {
                        let dummy_op_clk = op1.0 + table_len * (j + 1);
                        let dummy_op_addr = op1.1.get_address();
                        let dummy_op_value = op1.1.get_value();
                        dummy_ops.push((
                            dummy_op_clk,
                            Operation::DummyRead(dummy_op_addr, dummy_op_value),
                        ));
                    }
                }
            }
        }

        // TODO: Track number of operations at a given address instead of recounting here
        for (clk, dummy_op) in dummy_ops.iter() {
            let idx_addr = ops.binary_search_by_key(&dummy_op.get_address(), |(_, dummy_op)| {
                dummy_op.get_address()
            });
            if let Ok(idx_addr) = idx_addr {
                ops.insert(idx_addr, (*clk, dummy_op.clone()));
                let num_ops = ops[idx_addr..]
                    .iter()
                    .take_while(|(_, op2)| dummy_op.get_address() == op2.get_address())
                    .count();
                let idx_clk =
                    ops[idx_addr..(idx_addr + num_ops)].partition_point(|(clk2, _)| clk2 < clk);
                ops.insert(idx_addr + idx_clk, (*clk, *dummy_op));
            } else if let Err(idx_addr) = idx_addr {
                ops.insert(idx_addr, (*clk, dummy_op.clone()));
            }
        }

        // Pad the end of the table with dummy reads (to the next power of two)
        let num_dummy_ops = ops.len().next_power_of_two() - ops.len();
        let dummy_op_clk = ops.last().unwrap().0;
        let dummy_op_addr = ops.last().unwrap().1.get_address();
        let dummy_op_value = ops.last().unwrap().1.get_value();
        for _ in 0..num_dummy_ops {
            ops.push((
                dummy_op_clk,
                Operation::DummyRead(dummy_op_addr, dummy_op_value),
            ));
        }

        // Resort (TODO: this shouldn't be necessary if `insert_dummy_reads` is
        // implemented correctly...)
        ops.sort_by_key(|(clk, op)| (op.get_address(), *clk));
    }

    fn compute_address_diffs<F: PrimeField>(
        ops: Vec<(u32, Operation)>,
        rows: &mut Vec<[F; NUM_MEM_COLS]>,
    ) {
        if ops.is_empty() {
            return;
        }

        // Compute `diff` and `counter_mult`
        let mut diff = vec![F::zero(); rows.len()];
        let mut mult = vec![F::zero(); rows.len()];
        for n in 0..(rows.len() - 1) {
            let addr = ops[n].1.get_address();
            let addr_next = ops[n + 1].1.get_address();
            let value = if addr_next != addr {
                addr_next - addr
            } else {
                let clk = ops[n].0;
                let clk_next = ops[n + 1].0;
                clk_next - clk
            };
            diff[n] = F::from_canonical_u32(value);
            mult[value as usize] += F::one();
        }

        // Compute `diff_inv`
        let diff_inv = batch_multiplicative_inverse_allowing_zero(diff.clone());

        // Set trace values
        for n in 0..(rows.len() - 1) {
            rows[n][MEM_COL_MAP.diff] = diff[n];
            rows[n][MEM_COL_MAP.diff_inv] = diff_inv[n];
            rows[n][MEM_COL_MAP.counter_mult] = mult[n];

            let addr = ops[n].1.get_address();
            let addr_next = ops[n + 1].1.get_address();
            if addr_next - addr != 0 {
                rows[n][MEM_COL_MAP.addr_not_equal] = F::one();
            }
        }

        // The first row should have a zero-valued diff, which is "sent" to the local
        // range check bus. We need to account for that value on the receiving end here.
        rows[0][MEM_COL_MAP.counter_mult] += F::one();
    }
}
