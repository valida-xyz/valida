#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use p3_mersenne_31::Mersenne31 as Fp;
use valida_machine::Machine;
use valida_machine::Word;

pub mod columns;

enum Operation {
    Read(Fp, Word<Fp>),
    Write(Fp, Word<Fp>),
}

pub struct MemoryChip {
    cells: BTreeMap<Fp, Word<Fp>>,
    operations: BTreeMap<Fp, Vec<Operation>>,
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
