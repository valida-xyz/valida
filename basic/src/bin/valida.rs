use clap::Parser;
use std::io::{stdout, Write};

use valida_basic::BasicMachine;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{Machine, ProgramROM, StdinAdviceProvider};
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

    let mut machine = BasicMachine::<BabyBear>::default();
    let rom = match ProgramROM::from_file(&args.program) {
        Ok(contents) => contents,
        Err(e) => panic!("Failure to load file: {}. {}", &args.program, e),
    };
    machine.program_mut().set_program_rom(&rom);
    machine.cpu_mut().fp = args.stack_height;
    machine.cpu_mut().save_register_state();

    // Run the program
    machine.run(&rom, &mut StdinAdviceProvider);

    // Write output chip values to standard output
    stdout().write_all(&machine.output().bytes()).unwrap();
}
