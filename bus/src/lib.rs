use valida_machine::{BusArgument, Machine};

#[derive(Default)]
pub struct CpuMemBus {}

#[derive(Default)]
pub struct SharedCoprocessorBus {}

pub trait MachineWithGeneralBus: Machine {
    fn general_bus(&self) -> BusArgument;
}

pub trait MachineWithProgramBus: Machine {
    fn program_bus(&self) -> BusArgument;
}

pub trait MachineWithMemBus: Machine {
    fn mem_bus(&self) -> BusArgument;
}

pub trait MachineWithRangeBus8: Machine {
    fn range_bus(&self) -> BusArgument;
}

pub trait MachineWithPowerOfTwoBus: Machine {
    fn power_of_two_bus(&self) -> BusArgument;
}
