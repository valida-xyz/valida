use byteorder::{LittleEndian, ReadBytesExt};
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read, Result};

use valida_basic::BasicMachine;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{InstructionWord, Machine, Operands, ProgramROM, Word};
use valida_output::MachineWithOutputChip;

#[derive(Parser)]
struct Args {
    /// Program binary file
    #[arg(name = "FILE")]
    program: String,

    /// Stack height (which is also the initial frame pointer value)
    #[arg(long, default_value = "16777216")]
    stack_height: u32,
}

fn main() {
    let args = Args::parse();

    let mut machine = BasicMachine::default();
    let rom = load_program_rom(&args.program).unwrap();
    machine.cpu_mut().fp = args.stack_height;
    machine.cpu_mut().save_register_state();

    // Read standard input into the advice tape
    let mut input_bytes = Vec::new();
    std::io::stdin().read_to_end(&mut input_bytes).unwrap();
    let input_words = input_bytes
        .chunks(4)
        .map(|chunk| {
            let mut word = Word::default();
            chunk.iter().rev().enumerate().for_each(|(i, b)| {
                word[i] = *b;
            });
            word
        })
        .collect::<Vec<_>>();
    machine.cpu_mut().advice_tape.data.extend(input_words);

    // Run the program
    machine.run(rom);

    // Write output chip values to standard output
    for (_, byte) in machine.output().values.iter() {
        print!("{}", *byte as char);
    }
}

fn load_program_rom(filename: &str) -> Result<ProgramROM<i32>> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut instructions = Vec::new();

    while let Ok(opcode) = reader.read_u32::<LittleEndian>() {
        let mut operands_arr = [0i32; 5];
        for i in 0..5 {
            operands_arr[i] = reader.read_i32::<LittleEndian>()?;
        }
        let operands = Operands(operands_arr);
        instructions.push(InstructionWord { opcode, operands });
    }

    Ok(ProgramROM::new(instructions))
}
