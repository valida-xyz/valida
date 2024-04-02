#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use valida_machine::{ProgramROM, Word};

pub struct Program {
    code: ProgramROM<u32>,
    data: BTreeMap<u32, Word<u8>>,
}

pub fn load_elf_object_file(file: Vec<u8>) -> Program {
    todo!()
}
