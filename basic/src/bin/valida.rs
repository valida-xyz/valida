use clap::Parser;
use std::io::{stdin, stdout, Read, Write};

use valida_basic::BasicMachine;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{Machine, ProgramROM, Word, MEMORY_CELL_BYTES};
use valida_output::MachineWithOutputChip;
use valida_program::MachineWithProgramChip;

use p3_baby_bear::BabyBear;

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

    let mut machine = BasicMachine::<BabyBear, BabyBear>::default();
    let rom = ProgramROM::from_file(&args.program).unwrap();
    machine.program_mut().set_program_rom(&rom);
    machine.cpu_mut().fp = args.stack_height;
    machine.cpu_mut().save_register_state();

    // Read standard input into the advice tape
    let mut input_bytes = Vec::new();
    stdin().read_to_end(&mut input_bytes).unwrap();
    let input_words = input_bytes
        .chunks(MEMORY_CELL_BYTES)
        .map(|chunk| {
            let mut word = Word::default();
            (0..MEMORY_CELL_BYTES)
                .rev()
                .zip(chunk.iter())
                .for_each(|(i, b)| {
                    word[i] = *b;
                });
            word
        })
        .collect::<Vec<_>>();
    machine.cpu_mut().advice_tape.data.extend(input_words);

    // Run the program
    machine.run(&rom);

    // Write output chip values to standard output
    stdout().write_all(&machine.output().bytes()).unwrap();
}
