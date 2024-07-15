use p3_field::Field;
use valida_machine::{BusArgument, Machine};

#[derive(Default)]
pub struct CpuMemBus {}

#[derive(Default)]
pub struct SharedCoprocessorBus {}

pub trait MachineWithGeneralBus<F: Field>: Machine<F> {
    fn general_bus(&self) -> BusArgument;
}

pub trait MachineWithProgramBus<F: Field>: Machine<F> {
    fn program_bus(&self) -> BusArgument;
}

pub trait MachineWithMemBus<F: Field>: Machine<F> {
    fn mem_bus(&self) -> BusArgument;
}

pub trait MachineWithRangeBus8<F: Field>: Machine<F> {
    fn range_bus(&self) -> BusArgument;
}

pub trait MachineWithPowerOfTwoBus<F: Field>: Machine<F> {
    fn power_of_two_bus(&self) -> BusArgument;
}

pub trait MachineWithOutputBus<F: Field>: Machine<F> {
    fn output_bus(&self) -> BusArgument;
}
