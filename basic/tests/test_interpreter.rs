use p3_baby_bear::BabyBear;
use std::fs::File;
use std::io::BufWriter;
use std::io::Cursor;
use std::io::Read;
use std::process::{Command, Stdio};

use byteorder::{LittleEndian, WriteBytesExt};
use valida_machine::{InstructionWord, Operands, ProgramROM};
use valida_opcodes::{ADD32, BEQ, BNE, DIV32, IMM32, JAL, JALV, READ_ADVICE, STOP, STORE32, WRITE};

#[test]
fn run_fibonacci() {
    // Build the fibonacci binary
    let filepath = "tests/data/fibonacci.bin";
    let program_rom = build_fibonacci_program_rom();
    rom_to_bin(program_rom, filepath);

    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "valida", filepath])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    // Write the desired Fibonacci number to stdin
    let number = 25;
    let stdin = child.stdin.as_mut().expect("failed to get stdin");
    stdin.write_u32::<LittleEndian>(number).unwrap();

    // Compare stdout with the expected value in the Fibonacci sequence
    let value = fibonacci(number);
    let output = child.wait_with_output().expect("failed to wait on child");
    let mut cursor = Cursor::new(output.stdout);
    let mut buf = [0; 4];
    cursor.read_exact(&mut buf).unwrap();
    let result = u32::from_le_bytes(buf);
    assert_eq!(result, value);
}

fn fibonacci(n: u32) -> u32 {
    let mut a = 0u32;
    let mut b = 1u32;
    for _ in 0..n {
        let temp = a;
        a = b;
        (b, _) = temp.overflowing_add(b);
    }
    a
}

fn build_fibonacci_program_rom() -> ProgramROM<i32> {
    let mut program = vec![];

    // Label locations
    let fib_bb0 = 15;
    let fib_bb0_1 = 20;
    let fib_bb0_2 = 22;
    let fib_bb0_3 = 26;
    let fib_bb0_4 = 28;

    //main:                                   ; @main
    //; %bb.0:
    //	imm32	-4(fp), 0, 0, 0, 0
    //	imm32	-8(fp), 0, 0, 0, 10
    //	sw	-16(fp), -8(fp)
    //	imm32	-20(fp), 0, 0, 0, 28
    //	jal	-28(fp), fib, -28
    //	sw	-12(fp), -24(fp)
    //	sw	4(fp), -12(fp)
    //	exit
    //...
    program.extend([
        InstructionWord {
            opcode: IMM32,
            operands: Operands([-4, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: READ_ADVICE,
            operands: Operands([0, 1, -8, 0, 0]),
        },
        InstructionWord {
            opcode: STORE32,
            operands: Operands([0, -16, -8, 0, 0]),
        },
        InstructionWord {
            opcode: IMM32,
            operands: Operands([-20, 0, 0, 0, 28]),
        },
        InstructionWord {
            opcode: JAL,
            operands: Operands([-28, fib_bb0, -28, 0, 0]),
        },
        InstructionWord {
            opcode: STORE32,
            operands: Operands([0, -12, -24, 0, 0]),
        },
        InstructionWord {
            opcode: STORE32,
            operands: Operands([0, 4, -12, 0, 0]),
        },
        InstructionWord {
            opcode: WRITE,
            operands: Operands([0, 4, 0, 0, 1]),
        },
        InstructionWord {
            opcode: DIV32,
            operands: Operands([4, 4, 256, 0, 1]),
        },
        InstructionWord {
            opcode: WRITE,
            operands: Operands([0, 4, 0, 0, 1]),
        },
        InstructionWord {
            opcode: DIV32,
            operands: Operands([4, 4, 256, 0, 1]),
        },
        InstructionWord {
            opcode: WRITE,
            operands: Operands([0, 4, 0, 0, 1]),
        },
        InstructionWord {
            opcode: DIV32,
            operands: Operands([4, 4, 256, 0, 1]),
        },
        InstructionWord {
            opcode: WRITE,
            operands: Operands([0, 4, 0, 0, 1]),
        },
        InstructionWord {
            opcode: STOP,
            operands: Operands::default(),
        },
    ]);

    //fib:                                    ; @fib
    //; %bb.0:
    //	sw	-4(fp), 12(fp)
    //	imm32	-8(fp), 0, 0, 0, 0
    //	imm32	-12(fp), 0, 0, 0, 1
    //	imm32	-16(fp), 0, 0, 0, 0
    //	beq	.LBB0_1, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: STORE32,
            operands: Operands([0, -4, 12, 0, 0]),
        },
        InstructionWord {
            opcode: IMM32,
            operands: Operands([-8, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: IMM32,
            operands: Operands([-12, 0, 0, 0, 1]),
        },
        InstructionWord {
            opcode: IMM32,
            operands: Operands([-16, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: BEQ,
            operands: Operands([fib_bb0_1, 0, 0, 0, 0]),
        },
    ]);

    //.LBB0_1:
    //	bne	.LBB0_2, -16(fp), -4(fp)
    //	beq	.LBB0_4, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: BNE,
            operands: Operands([fib_bb0_2, -16, -4, 0, 0]),
        },
        InstructionWord {
            opcode: BEQ,
            operands: Operands([fib_bb0_4, 0, 0, 0, 0]),
        },
    ]);

    //; %bb.2:
    //	add	-20(fp), -8(fp), -12(fp)
    //	sw	-8(fp), -12(fp)
    //	sw	-12(fp), -20(fp)
    //	beq	.LBB0_3, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: ADD32,
            operands: Operands([-20, -8, -12, 0, 0]),
        },
        InstructionWord {
            opcode: STORE32,
            operands: Operands([0, -8, -12, 0, 0]),
        },
        InstructionWord {
            opcode: STORE32,
            operands: Operands([0, -12, -20, 0, 0]),
        },
        InstructionWord {
            opcode: BEQ,
            operands: Operands([fib_bb0_3, 0, 0, 0, 0]),
        },
    ]);

    //; %bb.3:
    //	addi	-16(fp), -16(fp), 1
    //	beq	.LBB0_1, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: ADD32,
            operands: Operands([-16, -16, 1, 0, 1]),
        },
        InstructionWord {
            opcode: BEQ,
            operands: Operands([fib_bb0_1, 0, 0, 0, 0]),
        },
    ]);

    //.LBB0_4:
    //	sw	4(fp), -8(fp)
    //	jalv	-4(fp), 0(fp), 8(fp)
    program.extend([
        InstructionWord {
            opcode: STORE32,
            operands: Operands([0, 4, -8, 0, 0]),
        },
        InstructionWord {
            opcode: JALV,
            operands: Operands([-4, 0, 8, 0, 0]),
        },
    ]);

    ProgramROM(program)
}

fn rom_to_bin(rom: ProgramROM<i32>, filepath: &str) {
    let mut writer = BufWriter::new(File::create(filepath).unwrap());
    for instruction in rom.0 {
        writer
            .write_u32::<LittleEndian>(instruction.opcode)
            .unwrap();
        for operand in instruction.operands.0.iter() {
            writer.write_i32::<LittleEndian>(*operand).unwrap();
        }
    }
}
