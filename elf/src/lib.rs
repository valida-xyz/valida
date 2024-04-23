#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use elf::abi;
use elf::endian::AnyEndian;
use elf::section::SectionHeader;
use elf::ElfBytes;
use valida_machine::{ProgramROM, Word, INSTRUCTION_ELEMENTS};

pub struct Program {
    pub code: ProgramROM<i32>,
    pub data: BTreeMap<u32, Word<u8>>,
    pub initial_program_counter: u32,
}

pub fn load_executable_file(file: Vec<u8>) -> Program {
    if file[0] == 0x7F && file[1] == 0x45 && file[2] == 0x4C && file[3] == 0x46 {
        load_elf_object_file(file)
    } else {
        Program {
            code: ProgramROM::from_machine_code(file.as_slice()),
            data: BTreeMap::new(),
            initial_program_counter: 0,
        }
    }
}

pub fn load_elf_object_file(file: Vec<u8>) -> Program {
    let file = ElfBytes::<AnyEndian>::minimal_parse(file.as_slice()).unwrap();
    let mut data_sections: Vec<(SectionHeader, &[u8])> = vec![];
    let mut bss_sections: Vec<SectionHeader> = vec![];
    let mut text_sections: Vec<(SectionHeader, &[u8])> = vec![];
    for section_header in file.section_headers().unwrap().iter() {
        let is_data: bool = section_header.sh_type == abi::SHT_PROGBITS
            && section_header.sh_flags == (abi::SHF_ALLOC | abi::SHF_WRITE).into();
        let is_rodata: bool = section_header.sh_type == abi::SHT_PROGBITS
            && (section_header.sh_flags == abi::SHF_ALLOC.into()
                || section_header.sh_flags == 0x32); // TODO: what is 0x32?
        let is_bss: bool = section_header.sh_type == abi::SHT_NOBITS
            && section_header.sh_flags == (abi::SHF_ALLOC | abi::SHF_WRITE).into();
        let is_text: bool = section_header.sh_type == abi::SHT_PROGBITS
            && section_header.sh_flags == (abi::SHF_ALLOC | abi::SHF_EXECINSTR).into();
        if is_data || is_rodata || is_text {
            let section_data = file.section_data(&section_header).unwrap();
            match section_data {
                (section_data, None) => {
                    if is_data || is_rodata {
                        data_sections.push((section_header, section_data));
                    } else if is_text {
                        text_sections.push((section_header, section_data));
                    }
                }
                _ => panic!("unsupported: compressed ELF section data"),
            }
        } else if is_bss {
            bss_sections.push(section_header);
        }
    }
    let initial_program_counter = text_sections
        .iter()
        .map(|(section_header, _)| section_header.sh_addr / ((INSTRUCTION_ELEMENTS * 4) as u64))
        .reduce(|a, b| a.min(b))
        .unwrap();
    let code_size = text_sections
        .iter()
        .map(|(section_header, _)| section_header.sh_addr + section_header.sh_size)
        .fold(0, |a, b| a.max(b));
    let mut code: Vec<u8> = vec![0; code_size as usize];
    for (section_header, section_data) in text_sections {
        for i in 0..section_header.sh_size as usize {
            code[i + section_header.sh_addr as usize] = section_data[i];
        }
    }
    let mut data: BTreeMap<u32, Word<u8>> = BTreeMap::new();
    for (section_header, section_data) in data_sections {
        let mut section_data = Vec::from(section_data);
        while section_data.len() % 4 != 0 {
            section_data.push(0);
        }
        for i in 0..(section_data.len() / 4) as usize {
            data.insert(
                <u64 as TryInto<u32>>::try_into(section_header.sh_addr).unwrap()
                    + <usize as TryInto<u32>>::try_into(i * 4).unwrap(),
                Word([
                    section_data[i * 4],
                    section_data[i * 4 + 1],
                    section_data[i * 4 + 2],
                    section_data[i * 4 + 3],
                ]),
            );
        }
    }
    Program {
        code: ProgramROM::from_machine_code(code.as_slice()),
        data,
        initial_program_counter: <u64 as TryInto<u32>>::try_into(initial_program_counter).unwrap(),
    }
}
