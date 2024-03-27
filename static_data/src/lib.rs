#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
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

impl<M, SC> Chip<M, SC> for StaticDataChip
where
    M: MachineWithMemBus<SC::Val>,
    SC: StarkConfig,
{
    // TODO
}
