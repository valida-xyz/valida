#![allow(non_snake_case)]

extern crate alloc;

use alloc::collections::BTreeMap;
use core::mem::transmute;
use p3_field::field::Field;
use valida_cpu::columns::CpuCols;
use valida_machine::{InstructionWord, Operands};
use valida_memory::columns::MemoryCols;

pub struct ProgramROM {
    data: Vec<InstructionWord<u32>>,
}

pub struct MachineTrace<T> {
    cpu: Vec<CpuCols<T>>,
    mem: Vec<MemoryCols<T>>,
}

impl<T: Copy> MachineTrace<T> {
    fn new() -> MachineTrace<T> {
        MachineTrace {
            cpu: Vec::new(),
            mem: Vec::new(),
        }
    }
}

impl ProgramROM {
    fn new() -> ProgramROM {
        ProgramROM { data: Vec::new() }
    }
}

// Read program ROM from file
fn read_program_rom(filename: &str) -> ProgramROM {
    todo!()
}
