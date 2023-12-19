use byteorder::{LittleEndian, WriteBytesExt};
use pest::Parser;
use pest_derive::*;
use std::collections::HashMap;
use valida_opcodes::*;

#[derive(Parser)]
#[grammar = "grammar/assembly.pest"]
pub struct AssemblyParser;

pub fn assemble(input: &str) -> Result<Vec<u8>, String> {
    let parsed = AssemblyParser::parse(Rule::assembly, input).unwrap();

    // First pass: Record label locations
    let mut label_locations = HashMap::new();
    let mut pc = 0;
    for pair in parsed.clone() {
        match pair.as_rule() {
            Rule::label => {
                let label_name = pair.as_str().trim().trim_end_matches(':');
                label_locations.insert(label_name, pc);
            }
            Rule::instruction => {
                pc += 1;
            }
            _ => {}
        }
    }

    // Second pass: Generate machine code and replace labels with PC locations
    let mut vec: Vec<u8> = Vec::new();
    for pair in parsed {
        match pair.as_rule() {
            Rule::instruction => {
                let mut inner_pairs = pair.into_inner();
                let mnemonic = inner_pairs.next().unwrap().as_str();
                let mut operands: Vec<i32> = inner_pairs
                    .filter_map(|p| {
                        if p.as_rule() == Rule::WHITESPACE {
                            return None;
                        }
                        let op_str = p.as_str();
                        let ret = if op_str.ends_with("(fp)") {
                            // Extract the numeric value from the string and convert to i32
                            op_str.trim_end_matches("(fp)").parse::<i32>().unwrap()
                        } else if label_locations.contains_key(op_str) {
                            // If operand is a label reference, replace with PC location
                            *label_locations.get(op_str).unwrap()
                        } else {
                            // Otherwise, use the operand as-is
                            op_str.parse::<i32>().unwrap()
                        };
                        Some(ret)
                    })
                    .collect();

                // Convert mnemonic to opcode
                let opcode = match mnemonic {
                    // Core CPU
                    "lw" => LOAD32,
                    "sw" => STORE32,
                    "jal" => JAL,
                    "jalv" => JALV,
                    "beq" | "beqi" => BEQ,
                    "bne" | "bnei" => BNE,
                    "imm32" => IMM32,
                    "stop" => STOP,

                    // Nondeterministic input
                    "advread" => READ_ADVICE,
                    "advwrite" => WRITE_ADVICE,

                    // U32 ALU
                    "add" | "addi" => ADD32,
                    "sub" | "subi" => SUB32,
                    "mul" | "muli" => MUL32,
                    "div" | "divi" => DIV32,
                    "sdiv"| "sdivi"=> SDIV32,
                    "lt" | "lti" => LT32,
                    "shl" | "shli" => SHL32,
                    "shr" | "shri" => SHR32,
                    "and" | "andi" => AND32,
                    "or" | "ori" => OR32,
                    "xor" | "xori" => XOR32,

                    // Native field
                    "feadd" => ADD,
                    "fesub" => SUB,
                    "femul" => MUL,

                    // Output
                    "write" => WRITE,

                    _ => panic!("Unknown mnemonic"),
                };

                // Insert zero operands if necessary
                match mnemonic {
                    "lw" => {
                        // (a, 0, c, 0, 0)
                        operands.insert(1, 0);
                        operands.extend(vec![0; 2]);
                    }
                    "sw" => {
                        // (0, b, c, 0, 0)
                        operands.insert(0, 0);
                        operands.extend(vec![0; 2]);
                    }
                    "imm32" | "write" => {
                        // (a, b, c, d, e)
                    }
                    "stop" => {
                        // (0, 0, 0, 0, 0)
                        operands.extend(vec![0; 5]);
                    }
                    "addi" | "subi" | "muli" | "divi" | "lti" | "shli" | "shri" | "beqi"
                    | "bnei" | "andi" | "ori" | "xori" => {
                        // (a, b, c, 0, 1)
                        operands.extend(vec![0, 1]);
                    }
                    _ => {
                        // (a, b, c, 0, 0)
                        operands.extend(vec![0; 2]);
                    }
                };

                // Write opcode and operands
                vec.write_u32::<LittleEndian>(opcode).unwrap();
                for operand in operands {
                    vec.write_i32::<LittleEndian>(operand).unwrap();
                }
            }
            _ => {}
        }
    }

    Ok(vec)
}
