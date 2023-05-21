use valida_machine::Machine;

#[derive(Default)]
pub struct CpuMemBus {}

#[derive(Default)]
pub struct SharedCoprocessorBus {}

pub trait MachineWithGeneralBus: Machine {
    fn general_bus(&self) -> usize;
}

pub trait MachineWithMemBus: Machine {
    fn mem_bus(&self) -> usize;
}
