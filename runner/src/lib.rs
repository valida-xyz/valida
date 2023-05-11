//extern crate alloc;
//
//use alloc::collections::BTreeMap;
//use core::mem::transmute;
//use p3_field::Field;
//use valida_cpu::columns::CpuCols;
//use p3_field::Field32;
//use valida_machine::Operands;
//use valida_memory::columns::MemoryCols;

//pub struct MachineTrace<T> {
//    cpu: Vec<CpuCols<T>>,
//    mem: Vec<MemoryCols<T>>,
//}
//
//impl<T: Copy> MachineTrace<T> {
//    fn new() -> MachineTrace<T> {
//        MachineTrace {
//            cpu: Vec::new(),
//            mem: Vec::new(),
//        }
//    }
//}
//
//impl ProgramROM {
//    fn new() -> ProgramROM {
//        ProgramROM { data: Vec::new() }
//    }
//}
//
//// Read program ROM from file
//fn read_program_rom(filename: &str) -> ProgramROM {
//    todo!()
//}
