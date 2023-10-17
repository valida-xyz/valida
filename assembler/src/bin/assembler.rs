use clap::{arg, command};
use std::fs::File;
use std::io::{self, Read, Write};
use valida_assembler::assemble;

fn main() {
    let matches = command!()
        .arg(arg!(
            -i --input <FILE> "The input assembly file to parse"
        ))
        .arg(arg!(
            -o --output <FILE> "The machine code output file"
        ))
        .get_matches();

    // Read assembly code from input file, or from stdin if no file is specified
    let mut assembly_code = String::new();
    if let Some(filepath) = matches.get_one::<String>("input") {
        std::fs::File::open(filepath)
            .expect("Failed to open input file")
            .read_to_string(&mut assembly_code)
            .expect("Failed to read from input file");
    } else {
        io::stdin()
            .read_to_string(&mut assembly_code)
            .expect("Failed to read from stdin");
    }

    // Write machine code to file, or stdout if no file is specified
    let machine_code = assemble(&assembly_code).expect("Failed to assemble code");
    if let Some(filepath) = matches.get_one::<String>("output") {
        File::create(filepath)
            .expect("Failed to open output file")
            .write_all(&machine_code)
            .expect("Failed to write to output file");
    } else {
        io::stdout()
            .write_all(&machine_code)
            .expect("Failed to write to stdout");
    }
}
