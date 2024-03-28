#![no_std]

extern crate alloc;

use crate::columns::{NUM_STATIC_DATA_COLS, STATIC_DATA_COL_MAP};
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use p3_air::VirtualPairCol;
use p3_field::{AbstractField, Field};
use p3_matrix::dense::RowMajorMatrix;
use valida_bus::MachineWithMemBus;
use valida_machine::{BusArgument, Chip, Interaction, Machine, StarkConfig, Word};

pub mod columns;
pub mod stark;

#[derive(Default)]
pub struct StaticDataChip {
    pub cells: BTreeMap<u32, Word<u8>>,
}

pub trait MachineWithStaticDataChip<F: Field>: Machine<F> {
    fn static_data(&self) -> &StaticDataChip;
    fn static_data_mut(&mut self) -> &mut StaticDataChip;
}

impl StaticDataChip {
    pub fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
        }
    }

    pub fn write(&mut self, address: u32, value: Word<u8>) {
        self.cells.insert(address, value);
    }
}

impl<M, SC> Chip<M, SC> for StaticDataChip
where
    M: MachineWithMemBus<SC::Val>,
    SC: StarkConfig,
{
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<SC::Val> {
        let mut rows = self.cells.iter()
            .map(|(addr, value)| {
                let mut row: Vec<SC::Val> = vec![SC::Val::from_canonical_u32(*addr)];
                row.extend(value.0.into_iter().map(SC::Val::from_canonical_u8).collect::<Vec<_>>());
                row
            })
            .flatten()
            .collect::<Vec<_>>();
        rows.resize(rows.len().next_power_of_two() * NUM_STATIC_DATA_COLS, SC::Val::zero());
        RowMajorMatrix::new(rows, NUM_STATIC_DATA_COLS)
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<SC::Val>> {
        let addr = VirtualPairCol::single_main(STATIC_DATA_COL_MAP.addr);
        let value = STATIC_DATA_COL_MAP.value.0.map(VirtualPairCol::single_main);
        let is_real_0 = VirtualPairCol::single_main(STATIC_DATA_COL_MAP.is_real);
        let is_real_1 = VirtualPairCol::single_main(STATIC_DATA_COL_MAP.is_real);
        let clk = VirtualPairCol::constant(SC::Val::zero());
        let mut fields = vec![is_real_0, clk, addr];
        fields.extend(value);
        let send = Interaction {
            fields,
            count: is_real_1,
            argument_index: machine.mem_bus(),
        };
        vec![send]
    }
}
