#![no_std]

extern crate alloc;

use crate::columns::NUM_PROGRAM_COLS;
use alloc::vec;
use alloc::vec::Vec;
use columns::{ProgramPreprocessedCols, NUM_PREPROCESSED_COLS};
use valida_bus::MachineWithProgramBus;
use valida_machine::{
    BusArgument, Chip, InstructionWord, Interaction, Machine, ProgramROM, ValidaPublicValues,
};
use valida_util::pad_to_power_of_two;

use p3_field::{AbstractField, Field};
use p3_matrix::{dense::RowMajorMatrix, Matrix, MatrixRows};
use valida_lookups::{LookupChip, LookupTable, LookupType};
use valida_machine::StarkConfig;

pub mod columns;
pub mod stark;

fn rom_to_table<F: Field>(rom: &ProgramROM<i32>) -> RowMajorMatrix<F> {
    // Pad the ROM to a power of two.
    let mut rom = rom.0.clone();
    let n = rom.len();
    rom.resize(n.next_power_of_two(), InstructionWord::default());

    let flattened = rom
        .into_iter()
        .enumerate()
        .flat_map(|(n, word)| {
            let mut row = vec![F::zero(); NUM_PREPROCESSED_COLS];
            row[0] = F::from_canonical_usize(n);
            row[1..].copy_from_slice(&word.flatten());
            row
        })
        .collect();
    RowMajorMatrix::new(flattened, NUM_PREPROCESSED_COLS)
}

#[derive(Default)]
pub struct ProgramTablePublic {
    rom: ProgramROM<i32>,
}

#[derive(Default)]
pub struct ProgramTablePreprocessed {
    rom: ProgramROM<i32>,
}

impl<F> LookupTable<F> for ProgramTablePublic
where
    F: Field,
{
    type M<'a> = RowMajorMatrix<F>;

    fn lookup_type(&self) -> LookupType {
        LookupType::Public
    }

    fn table(&self) -> RowMajorMatrix<F> {
        rom_to_table(&self.rom)
    }
}

impl<F: Field> LookupTable<F> for ProgramTablePreprocessed {
    type M<'a> = RowMajorMatrix<F>;

    fn lookup_type(&self) -> LookupType {
        LookupType::Preprocessed
    }
    fn table(&self) -> RowMajorMatrix<F> {
        rom_to_table(&self.rom)
    }
}

pub trait ProgramChipTrait<F> {
    fn program_rom(&self) -> &ProgramROM<i32>;
    fn set_program_rom(&mut self, rom: &ProgramROM<i32>);
}

pub type ProgramChip<F> = LookupChip<ProgramTablePublic, F>;
//type ProgramChip<F> = LookupChip<ProgramTablePreprocessed, F>;

impl<F: Field> ProgramChipTrait<i32> for ProgramChip<F> {
    fn program_rom(&self) -> &ProgramROM<i32> {
        &self.table.rom
    }

    fn set_program_rom(&mut self, rom: &ProgramROM<i32>) {
        self.table.rom = rom.clone();
        self.counts = vec![0; self.table().height()];
    }
}

// #[derive(Default)]
// pub struct ProgramChip {
//     pub program_rom: ProgramROM<i32>,
//     pub counts: Vec<u32>,
// }

// impl ProgramChip {
//     pub fn set_program_rom(&mut self, rom: &ProgramROM<i32>) {
//         let counts = vec![0; rom.0.len()];
//         self.program_rom = rom.clone();
//         self.counts = counts;
//     }
// }

// impl<M, SC> Chip<M, SC> for ProgramChip
// where
//     M: MachineWithProgramBus<SC::Val>,
//     SC: StarkConfig,
// {
//     type Public = ValidaPublicValues<SC::Val>;

//     fn generate_trace(&self, _machine: &M) -> RowMajorMatrix<SC::Val> {
//         let mut values = self
//             .counts
//             .iter()
//             .map(|c| SC::Val::from_canonical_u32(*c))
//             .collect();

//         pad_to_power_of_two::<NUM_PROGRAM_COLS, SC::Val>(&mut values);

//         RowMajorMatrix::new(values, NUM_PROGRAM_COLS)
//     }

//     fn global_receives(&self, _machine: &M) -> Vec<Interaction<SC::Val>> {
//         // let pc = VirtualPairCol::single_preprocessed(PREPROCESSED_COL_MAP.pc);
//         // let opcode = VirtualPairCol::single_preprocessed(PREPROCESSED_COL_MAP.opcode);
//         // let mut fields = vec![pc, opcode];
//         // fields.extend(
//         //     PREPROCESSED_COL_MAP
//         //         .operands
//         //         .0
//         //         .iter()
//         //         .map(|op| VirtualPairCol::single_preprocessed(*op)),
//         // );
//         // let receives = Interaction {
//         //     fields,
//         //     count: VirtualPairCol::single_main(COL_MAP.multiplicity),
//         //     argument_index: machine.program_bus(),
//         // };
//         // vec![receives]
//         vec![]
//     }
// }

pub trait MachineWithProgramChip<F: Field>: Machine<F> {
    fn program(&self) -> &ProgramChip<F>;

    fn program_mut(&mut self) -> &mut ProgramChip<F>;

    /// Read a word from the program code, and update the associated counter.
    fn read_word(&mut self, index: usize) {
        assert!(index < self.program().table().height());
        self.program_mut().counts[index] += 1;
    }
}
