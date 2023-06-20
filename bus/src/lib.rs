use valida_machine::{BusArgument, Machine};

#[derive(Default)]
pub struct CpuMemBus {}

#[derive(Default)]
pub struct SharedCoprocessorBus {}

pub trait MachineWithGeneralBus: Machine {
    fn general_bus(&self) -> BusArgument;
}

pub trait MachineWithMemBus: Machine {
    fn mem_bus(&self) -> BusArgument;
}

pub trait MachineWithRangeBus: Machine {
    fn range_bus(&self) -> BusArgument;
}
