use core::fmt::Debug;
use p3_field::Field;
use proptest::prelude::Arbitrary;
use valida_machine::{BusArgument, Machine};

#[derive(Default)]
pub struct CpuMemBus {}

#[derive(Default)]
pub struct SharedCoprocessorBus {}

pub trait MachineWithGeneralBus<F: Field + Arbitrary + Debug>: Machine<F> {
    fn general_bus(&self) -> BusArgument;
}

pub trait MachineWithProgramBus<F: Field + Arbitrary + Debug>: Machine<F> {
    fn program_bus(&self) -> BusArgument;
}

pub trait MachineWithMemBus<F: Field + Arbitrary + Debug>: Machine<F> {
    fn mem_bus(&self) -> BusArgument;
}

pub trait MachineWithRangeBus8<F: Field + Arbitrary + Debug>: Machine<F> {
    fn range_bus(&self) -> BusArgument;
}

pub trait MachineWithPowerOfTwoBus<F: Field + Arbitrary + Debug>: Machine<F> {
    fn power_of_two_bus(&self) -> BusArgument;
}
