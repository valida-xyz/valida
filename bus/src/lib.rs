use valida_machine::{BusArgumentIndex, Machine};

#[derive(Default)]
pub struct CpuMemBus {}

#[derive(Default)]
pub struct SharedCoprocessorBus {}

pub trait MachineWithGeneralBus: Machine {
    fn general_bus(&self) -> BusArgumentIndex;
}

pub trait MachineWithMemBus: Machine {
    fn mem_bus(&self) -> BusArgumentIndex;
}
