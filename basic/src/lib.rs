#![no_std]
#![allow(unused)]

extern crate alloc;
use alloc::vec::Vec;
use core::marker::PhantomData;
use p3_field::{extension::BinomialExtensionField, TwoAdicField};
use p3_goldilocks::Goldilocks;
use valida_alu_u32::{
    add::{Add32Chip, Add32Instruction, MachineWithAdd32Chip},
    bitwise::{
        And32Instruction, Bitwise32Chip, MachineWithBitwise32Chip, Or32Instruction,
        Xor32Instruction,
    },
    div::{Div32Chip, Div32Instruction, MachineWithDiv32Chip},
    lt::{Lt32Chip, Lt32Instruction, MachineWithLt32Chip},
    mul::{MachineWithMul32Chip, Mul32Chip, Mul32Instruction},
    shift::{MachineWithShift32Chip, Shift32Chip, Shl32Instruction, Shr32Instruction},
    sub::{MachineWithSub32Chip, Sub32Chip, Sub32Instruction},
};
use valida_bus::{
    MachineWithGeneralBus, MachineWithMemBus, MachineWithProgramBus, MachineWithRangeBus8,
};
use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    Load32Instruction, ReadAdviceInstruction, StopInstruction, Store32Instruction,
};
use valida_cpu::{CpuChip, MachineWithCpuChip};
use valida_derive::Machine;
use valida_machine::{
    config::StarkConfig, proof::MachineProof, AbstractExtensionField, AbstractField, BusArgument,
    Chip, ExtensionField, Instruction, Machine, PrimeField64, ProgramROM, ValidaAirBuilder,
};
use valida_memory::{MachineWithMemoryChip, MemoryChip};
use valida_output::{MachineWithOutputChip, OutputChip, WriteInstruction};
use valida_program::{MachineWithProgramChip, ProgramChip};
use valida_range::{MachineWithRangeChip, RangeCheckerChip};

use p3_maybe_rayon::*;

#[derive(Default)]
pub struct BasicMachine<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> {
    // Core instructions
    load32: Load32Instruction,

    store32: Store32Instruction,

    jal: JalInstruction,

    jalv: JalvInstruction,

    beq: BeqInstruction,

    bne: BneInstruction,

    imm32: Imm32Instruction,

    stop: StopInstruction,

    // ALU instructions
    add32: Add32Instruction,

    sub32: Sub32Instruction,

    mul32: Mul32Instruction,

    div32: Div32Instruction,

    shl32: Shl32Instruction,

    shr32: Shr32Instruction,

    lt32: Lt32Instruction,

    and32: And32Instruction,

    or32: Or32Instruction,

    xor32: Xor32Instruction,

    // Input/output instructions
    read: ReadAdviceInstruction,

    write: WriteInstruction,

    cpu: CpuChip,

    program: ProgramChip,

    mem: MemoryChip,

    add_u32: Add32Chip,

    sub_u32: Sub32Chip,

    mul_u32: Mul32Chip,

    div_u32: Div32Chip,

    shift_u32: Shift32Chip,

    lt_u32: Lt32Chip,

    bitwise_u32: Bitwise32Chip,

    output: OutputChip,

    range: RangeCheckerChip<256>,

    _phantom_base: core::marker::PhantomData<F>,
    _phantom_extension: core::marker::PhantomData<EF>,
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithGeneralBus
    for BasicMachine<F, EF>
{
    fn general_bus(&self) -> BusArgument {
        BusArgument::Global(0)
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithProgramBus
    for BasicMachine<F, EF>
{
    fn program_bus(&self) -> BusArgument {
        BusArgument::Global(1)
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithMemBus
    for BasicMachine<F, EF>
{
    fn mem_bus(&self) -> BusArgument {
        BusArgument::Global(2)
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithRangeBus8
    for BasicMachine<F, EF>
{
    fn range_bus(&self) -> BusArgument {
        BusArgument::Global(3)
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithCpuChip
    for BasicMachine<F, EF>
{
    fn cpu(&self) -> &CpuChip {
        &self.cpu
    }

    fn cpu_mut(&mut self) -> &mut CpuChip {
        &mut self.cpu
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithProgramChip
    for BasicMachine<F, EF>
{
    fn program(&self) -> &ProgramChip {
        &self.program
    }

    fn program_mut(&mut self) -> &mut ProgramChip {
        &mut self.program
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithMemoryChip
    for BasicMachine<F, EF>
{
    fn mem(&self) -> &MemoryChip {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut MemoryChip {
        &mut self.mem
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithAdd32Chip
    for BasicMachine<F, EF>
{
    fn add_u32(&self) -> &Add32Chip {
        &self.add_u32
    }

    fn add_u32_mut(&mut self) -> &mut Add32Chip {
        &mut self.add_u32
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithSub32Chip
    for BasicMachine<F, EF>
{
    fn sub_u32(&self) -> &Sub32Chip {
        &self.sub_u32
    }

    fn sub_u32_mut(&mut self) -> &mut Sub32Chip {
        &mut self.sub_u32
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithMul32Chip
    for BasicMachine<F, EF>
{
    fn mul_u32(&self) -> &Mul32Chip {
        &self.mul_u32
    }

    fn mul_u32_mut(&mut self) -> &mut Mul32Chip {
        &mut self.mul_u32
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithDiv32Chip
    for BasicMachine<F, EF>
{
    fn div_u32(&self) -> &Div32Chip {
        &self.div_u32
    }

    fn div_u32_mut(&mut self) -> &mut Div32Chip {
        &mut self.div_u32
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithBitwise32Chip
    for BasicMachine<F, EF>
{
    fn bitwise_u32(&self) -> &Bitwise32Chip {
        &self.bitwise_u32
    }

    fn bitwise_u32_mut(&mut self) -> &mut Bitwise32Chip {
        &mut self.bitwise_u32
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithLt32Chip
    for BasicMachine<F, EF>
{
    fn lt_u32(&self) -> &Lt32Chip {
        &self.lt_u32
    }

    fn lt_u32_mut(&mut self) -> &mut Lt32Chip {
        &mut self.lt_u32
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithShift32Chip
    for BasicMachine<F, EF>
{
    fn shift_u32(&self) -> &Shift32Chip {
        &self.shift_u32
    }

    fn shift_u32_mut(&mut self) -> &mut Shift32Chip {
        &mut self.shift_u32
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithOutputChip
    for BasicMachine<F, EF>
{
    fn output(&self) -> &OutputChip {
        &self.output
    }

    fn output_mut(&mut self) -> &mut OutputChip {
        &mut self.output
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> MachineWithRangeChip<256>
    for BasicMachine<F, EF>
{
    fn range(&self) -> &RangeCheckerChip<256> {
        &self.range
    }

    fn range_mut(&mut self) -> &mut RangeCheckerChip<256> {
        &mut self.range
    }
}

impl<F: PrimeField64 + TwoAdicField, EF: ExtensionField<F>> Machine for BasicMachine<F, EF> {
    type F = F;
    type EF = EF;
    fn run(&mut self, program: &ProgramROM<i32>) {
        loop {
            let pc = self.cpu().pc;
            let instruction = program.get_instruction(pc);
            let opcode = instruction.opcode;
            let ops = instruction.operands;
            match opcode {
                <Load32Instruction as Instruction<Self>>::OPCODE => {
                    Load32Instruction::execute(self, ops);
                }
                <JalInstruction as Instruction<Self>>::OPCODE => {
                    JalInstruction::execute(self, ops);
                }
                <JalvInstruction as Instruction<Self>>::OPCODE => {
                    JalvInstruction::execute(self, ops);
                }
                <BeqInstruction as Instruction<Self>>::OPCODE => {
                    BeqInstruction::execute(self, ops);
                }
                <BneInstruction as Instruction<Self>>::OPCODE => {
                    BneInstruction::execute(self, ops);
                }
                <Imm32Instruction as Instruction<Self>>::OPCODE => {
                    Imm32Instruction::execute(self, ops);
                }
                <Add32Instruction as Instruction<Self>>::OPCODE => {
                    Add32Instruction::execute(self, ops);
                }
                <Sub32Instruction as Instruction<Self>>::OPCODE => {
                    Sub32Instruction::execute(self, ops);
                }
                <Mul32Instruction as Instruction<Self>>::OPCODE => {
                    Mul32Instruction::execute(self, ops);
                }
                <Div32Instruction as Instruction<Self>>::OPCODE => {
                    Div32Instruction::execute(self, ops);
                }
                <Shl32Instruction as Instruction<Self>>::OPCODE => {
                    Shl32Instruction::execute(self, ops);
                }
                <Shr32Instruction as Instruction<Self>>::OPCODE => {
                    Shr32Instruction::execute(self, ops);
                }
                <Lt32Instruction as Instruction<Self>>::OPCODE => {
                    Lt32Instruction::execute(self, ops);
                }
                <And32Instruction as Instruction<Self>>::OPCODE => {
                    And32Instruction::execute(self, ops);
                }
                <Or32Instruction as Instruction<Self>>::OPCODE => {
                    Or32Instruction::execute(self, ops);
                }
                <Xor32Instruction as Instruction<Self>>::OPCODE => {
                    Xor32Instruction::execute(self, ops);
                }
                <ReadAdviceInstruction as Instruction<Self>>::OPCODE => {
                    ReadAdviceInstruction::execute(self, ops);
                }
                <WriteInstruction as Instruction<Self>>::OPCODE => {
                    WriteInstruction::execute(self, ops);
                }
                <StopInstruction as Instruction<Self>>::OPCODE => {
                    StopInstruction::execute(self, ops);
                }
                _ => {}
            }

            self.read_word(pc as usize);

            if opcode == <StopInstruction as Instruction<Self>>::OPCODE {
                break;
            }
        }
        let n = self.cpu().clock.next_power_of_two() - self.cpu().clock;
        for _ in 0..n {
            self.read_word(self.cpu().pc as usize);
        }
    }

    fn prove<SC>(&self, config: &SC) -> MachineProof<SC>
    where
        SC: StarkConfig<Val = Self::F, Challenge = Self::EF>,
    {
        MachineProof {
            chip_proofs: Vec::new(),
            phantom: PhantomData::default(),
        }
    }

    fn verify<SC>(proof: &MachineProof<SC>) -> Result<(), ()>
    where
        SC: StarkConfig<Val = Self::F, Challenge = Self::EF>,
    {
        Ok(())
    }
}
