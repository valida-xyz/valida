#![allow(non_snake_case)]

use core::mem::transmute;
use p3_field::field::Field;
use valida_cpu::{columns::CpuCols, INSTRUCTION_ELEMENTS, OPERAND_ELEMENTS};
use valida_machine::{Addressable, Word};
use valida_memory::{columns::MemoryCols, Memory};

mod cpu;

enum Opcode {
    // CPU
    Load32,
    Store32,
    Jal,
    Jalv,
    Beq,
    Bne,
    Imm32,
}

pub struct Operands {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
}

impl Operands {
    fn as_slice<F>(&self) -> &[F] {
        todo!()
    }
}

pub struct ProgramROM {
    data: Vec<[u32; INSTRUCTION_ELEMENTS]>,
}

pub struct MachineTrace<T: Copy> {
    cpu: Vec<CpuCols<T>>,
    mem: Vec<MemoryCols<T>>,
}

pub struct MachineState<F: Copy> {
    pc: u32,
    fp: u32,
    memory: Memory<F>,
    program_rom: ProgramROM,

    trace: MachineTrace<F>,
}

fn decode_raw_instr(values: [u32; INSTRUCTION_ELEMENTS]) -> (Opcode, Operands) {
    let (instr, operands) =
        unsafe { transmute::<[u32; INSTRUCTION_ELEMENTS], (u32, Operands)>(values) };
    let opcode = match instr {
        0x01 => Opcode::Load32,
        0x02 => Opcode::Store32,
        0x03 => Opcode::Jal,
        0x04 => Opcode::Jalv,
        0x05 => Opcode::Beq,
        0x06 => Opcode::Bne,
        0x07 => Opcode::Imm32,
        _ => panic!("Unknown operation"),
    };
    (opcode, operands)
}

impl<F: Field + Addressable<F>> MachineState<F> {
    fn new(pc: u32, fp: u32) -> MachineState<F> {
        MachineState {
            pc,
            fp,
            memory: Memory::new(),
            program_rom: ProgramROM::new(),
            trace: MachineTrace::new(),
        }
    }

    fn transition(&mut self) {
        let mut cpu_row: CpuCols<F> = CpuCols::default();
        let mut mem_rows: Vec<MemoryCols<F>> = Vec::with_capacity(3);

        let instruction = self.program_rom.data[self.pc as usize];
        let (opcode, operands) = decode_raw_instr(instruction);
        self.execute(opcode, operands, &mut cpu_row, &mut mem_rows);

        self.trace.cpu.push(cpu_row);
        self.trace.mem.append(&mut mem_rows);
    }

    fn execute(
        &mut self,
        opcode: Opcode,
        ops: Operands,
        cpu_row: &mut CpuCols<F>,
        mem_rows: &mut Vec<MemoryCols<F>>,
    ) {
        match opcode {
            Opcode::Load32 => cpu::load32(self, ops, cpu_row, mem_rows),
            Opcode::Store32 => cpu::store32(self, ops, cpu_row, mem_rows),
            Opcode::Jal => {}
            Opcode::Jalv => {}
            Opcode::Beq => {}
            Opcode::Bne => {}
            Opcode::Imm32 => {}
        }
    }
}

impl<T: Copy> MachineTrace<T> {
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
