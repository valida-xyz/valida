#![allow(non_snake_case)]

use p3_field::field::Field;
use valida_cpu::{columns::CpuCols, Cpu, INSTRUCTION_ELEMENTS};
use valida_memory::{columns::MemoryCols, Memory};

enum Operation<F: Field> {
    // CPU
    Load32([F; 2]),
    Store32([F; 2]),
    Jal([F; 3]),
    Jalv([F; 3]),
    Beq([F; 3]),
    Bne([F; 3]),
    Imm32([F; 5]),
}

pub struct ProgramROM {
    data: Vec<[u32; INSTRUCTION_ELEMENTS]>,
}

pub struct MachineTrace<T> {
    cpu: Vec<CpuCols<T>>,
    mem: Vec<MemoryCols<T>>,
}

pub struct MachineState<F: Field> {
    pc: usize,
    fp: usize,
    memory: Memory<F>,
    program_rom: ProgramROM,

    trace: MachineTrace<F>,
}

impl<F: Field> From<[u32; INSTRUCTION_ELEMENTS]> for Operation<F> {
    fn from(values: [u32; INSTRUCTION_ELEMENTS]) -> Self {
        todo!("Implement From<u32> for Field");
        //let opcode = values[0];
        //let op_a = values[1];
        //let op_b = values[2];
        //let op_c = values[3];
        //let is_imm = values[4];
        //match opcode {
        //    0x01 => Operation::LOAD32([op_a, op_c]),
        //    0x02 => Operation::STORE32([op_b, op_c]),
        //    0x03 => Operation::JAL([op_a, op_b, op_c]),
        //    0x04 => Operation::JALV([op_a, op_b, op_c]),
        //    0x05 => Operation::BEQ([op_a, op_b, op_c]),
        //    0x06 => Operation::BNE([op_a, op_b, op_c]),
        //    0x07 => Operation::IMM32(values[1..].try_into().unwrap()),
        //    _ => panic!("Unknown operation"),
        //}
    }
}

impl<F: Field> MachineState<F> {
    fn new(pc: usize, fp: usize) -> MachineState<F> {
        MachineState {
            pc,
            fp,
            memory: Memory::new(),
            program_rom: ProgramROM::new(),
            trace: MachineTrace::new(),
        }
    }

    fn transition(&mut self) {
        let mut row: CpuCols<F> = CpuCols::default();

        let instruction = self.program_rom.data[self.pc];
        let op = Operation::from(instruction);
        self.execute_op(op, row);
    }

    fn execute_op(&mut self, op: Operation<F>, mut row: CpuCols<F>) {
        match op {
            Load32 => Cpu::load32(row),

            Store32 => {
                // ...
            }
            Jal => {
                // ...
            }
            Jalv => {
                // ...
            }
            Beq => {
                // ...
            }
            Bne => {
                // ...
            }
            Imm32 => {
                // ...
            }
        }
    }
}

impl<T> MachineTrace<T> {
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

// Parse program ROM into operations
fn parse_program_rom<F: Field>(program_rom: &ProgramROM) -> Vec<Operation<F>> {
    todo!()
}

// Test
#[test]
fn load_program() {
    let mut machine = Machine::new();
    machine.load_program(vec![0x00000000, 0x00000001, 0x00000002, 0x00000003]);
    machine.run();
}
