use crate::{AdviceProvider, Machine, Word, INSTRUCTION_ELEMENTS, OPERAND_ELEMENTS};
use byteorder::{ByteOrder, LittleEndian};
use p3_field::Field;

use valida_opcodes::{Opcode, IMM32};

pub trait Instruction<M: Machine<F>, F: Field> {
    const OPCODE: u32;

    fn execute(state: &mut M, ops: Operands<i32>);

    fn execute_with_advice<Adv: AdviceProvider>(
        state: &mut M,
        ops: Operands<i32>,
        _advice: &mut Adv,
    ) {
        Self::execute(state, ops)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct InstructionWord<F> {
    pub opcode: u32,
    pub operands: Operands<F>,
}

impl ToString for InstructionWord<i32> {
    fn to_string(&self) -> String {
        let opcode = match Opcode::try_from(self.opcode) {
            Ok(opcode_name) => {
                format!("{:?}", opcode_name)
            }
            Err(_) => {format!("UNKNOWN_OP:{}", self.opcode.to_string())}
        };
        format!("{} {}", opcode, self.print_operands())
    }
}

impl InstructionWord<i32> {
    pub fn flatten<F: Field>(&self) -> [F; INSTRUCTION_ELEMENTS] {
        let mut result = [F::default(); INSTRUCTION_ELEMENTS];
        result[0] = F::from_canonical_u32(self.opcode);
        result[1..].copy_from_slice(&Operands::<F>::from_i32_slice(&self.operands.0).0);
        result
    }

    pub fn print_imm32(&self) -> String {
        assert!(self.opcode == IMM32, "Instruction is not immediate");
    
        //extract the immediate value
        let imm0 = self.operands.0[1];
        let imm1 = self.operands.0[2];
        let imm2 = self.operands.0[3];
        let imm3 = self.operands.0[4];
        format!("{}(fp), {}", self.operands.0[0], imm0 << 24 | imm1 << 16 | imm2 << 8 | imm3)
    }

    pub fn print_first_operand(&self) -> String {
        format!("{}(fp)", self.operands.0[1])
    }

    pub fn print_second_operand(&self) -> String {
        let second_opnd_is_imm = self.operands.0[4] != 0;
        if second_opnd_is_imm {
            format!("{}", self.operands.0[2])
        } else {
            format!("{}(fp)", self.operands.0[2])
        }
    }

    pub fn print_address(&self, index : usize) -> String {
        format!("{}", self.operands.0[index]/24)
    }

    pub fn print_operands(&self) -> String {
        match self.opcode {
            valida_opcodes::IMM32 => self.print_imm32(),
            valida_opcodes::JAL =>
                format!(
                    "{}(fp), PC: {}, {}",
                    self.operands.0[0],
                    self.print_address(1),
                    self.operands.0[2]),
            valida_opcodes::JALV =>
                format!(
                    "{}(fp), {}(fp), {}(fp)",
                    self.operands.0[0],
                    self.operands.0[1],
                    self.operands.0[2]),
            valida_opcodes::LOADFP =>
                format!(
                    "{}(fp), {}",
                    self.operands.0[0],
                    self.operands.0[1]),
            valida_opcodes::BEQ |
            valida_opcodes::BNE =>
                format!(
                    "{}, {}, {}",
                    self.print_address(0),
                    self.print_first_operand(),
                    self.print_second_operand()),
            valida_opcodes::STOP => "".to_string(),
            valida_opcodes::LOAD32 =>
                format!(
                    "{}(fp), {}(fp)",
                    self.operands.0[0],
                    self.operands.0[1]),
            valida_opcodes::STORE32 =>
                format!(
                    "{}(fp), {}(fp)",
                    self.operands.0[1],
                    self.operands.0[2]),
            _ => {
                format!(
                    "{}(fp), {}, {}", self.operands.0[0], self.print_first_operand(), self.print_second_operand()
                )
            }
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Operands<F>(pub [F; OPERAND_ELEMENTS]);

impl<F: Copy> Operands<F> {
    pub fn a(&self) -> F {
        self.0[0]
    }
    pub fn b(&self) -> F {
        self.0[1]
    }
    pub fn c(&self) -> F {
        self.0[2]
    }
    pub fn d(&self) -> F {
        self.0[3]
    }
    pub fn e(&self) -> F {
        self.0[4]
    }
    pub fn is_imm(&self) -> F {
        self.0[4]
    }
    pub fn imm32(&self) -> Word<F> {
        Word([self.0[1], self.0[2], self.0[3], self.0[4]])
    }
}

impl<F: Field> Operands<F> {
    pub fn from_i32_slice(slice: &[i32]) -> Self {
        let mut operands = [F::zero(); OPERAND_ELEMENTS];
        for (i, &operand) in slice.iter().enumerate() {
            let abs = F::from_canonical_u32(operand.abs() as u32);
            operands[i] = if operand < 0 { -abs } else { abs };
        }
        Self(operands)
    }
}

#[derive(Default, Clone)]
pub struct ProgramROM<F>(pub Vec<InstructionWord<F>>);

impl<F> ProgramROM<F> {
    pub fn new(instructions: Vec<InstructionWord<F>>) -> Self {
        Self(instructions)
    }

    pub fn get_instruction(&self, pc: u32) -> &InstructionWord<F> {
        &self.0[pc as usize]
    }
}

impl ProgramROM<i32> {
    pub fn from_machine_code(mc: &[u8]) -> Self {
        let mut instructions = Vec::new();
        for chunk in mc.chunks_exact(INSTRUCTION_ELEMENTS * 4) {
            instructions.push(InstructionWord {
                opcode: LittleEndian::read_u32(&chunk[0..4]),
                operands: Operands([
                    LittleEndian::read_i32(&chunk[4..8]),
                    LittleEndian::read_i32(&chunk[8..12]),
                    LittleEndian::read_i32(&chunk[12..16]),
                    LittleEndian::read_i32(&chunk[16..20]),
                    LittleEndian::read_i32(&chunk[20..24]),
                ]),
            });
        }
        Self(instructions)
    }

    #[cfg(feature = "std")]
    pub fn from_file(filename: &str) -> std::io::Result<Self> {
        use byteorder::ReadBytesExt;
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        let mut instructions = Vec::new();

        while let Ok(opcode) = reader.read_u32::<LittleEndian>() {
            let mut operands_arr = [0i32; OPERAND_ELEMENTS];
            for i in 0..OPERAND_ELEMENTS {
                operands_arr[i] = reader.read_i32::<LittleEndian>()?;
            }
            let operands = Operands(operands_arr);
            instructions.push(InstructionWord { opcode, operands });
        }

        Ok(ProgramROM::new(instructions))
    }
}
