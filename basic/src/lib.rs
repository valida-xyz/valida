#![no_std]
#![allow(unused)]

extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use p3_air::Air;
use p3_commit::{Pcs, UnivariatePcs, UnivariatePcsWithLde};
use p3_field::PrimeField32;
use p3_field::{extension::BinomialExtensionField, TwoAdicField};
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::*;
use p3_util::log2_ceil_usize;
use valida_alu_u32::{
    add::{Add32Chip, Add32Instruction, MachineWithAdd32Chip},
    bitwise::{
        And32Instruction, Bitwise32Chip, MachineWithBitwise32Chip, Or32Instruction,
        Xor32Instruction,
    },
    com::{Com32Chip, Eq32Instruction, MachineWithCom32Chip, Ne32Instruction},
    div::{Div32Chip, Div32Instruction, MachineWithDiv32Chip, SDiv32Instruction},
    lt::{Lt32Chip, Lt32Instruction, Lte32Instruction, MachineWithLt32Chip},
    mul::{
        MachineWithMul32Chip, Mul32Chip, Mul32Instruction, Mulhs32Instruction, Mulhu32Instruction,
    },
    shift::{
        MachineWithShift32Chip, Shift32Chip, Shl32Instruction, Shr32Instruction, Sra32Instruction,
    },
    sub::{MachineWithSub32Chip, Sub32Chip, Sub32Instruction},
};
use valida_bus::{
    MachineWithGeneralBus, MachineWithMemBus, MachineWithProgramBus, MachineWithRangeBus8,
};
use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    Load32Instruction, LoadFpInstruction, ReadAdviceInstruction, StopInstruction,
    Store32Instruction,
};
use valida_cpu::{CpuChip, MachineWithCpuChip};
use valida_machine::{
    AdviceProvider, BusArgument, Chip, ChipProof, Instruction, Machine, MachineProof, ProgramROM,
    ValidaAirBuilder,
};
use valida_memory::{MachineWithMemoryChip, MemoryChip};
use valida_output::{MachineWithOutputChip, OutputChip, WriteInstruction};
use valida_program::{MachineWithProgramChip, ProgramChip};
use valida_range::{MachineWithRangeChip, RangeCheckerChip};
use valida_static_data::{MachineWithStaticDataChip, StaticDataChip};

use p3_maybe_rayon::prelude::*;
use valida_machine::StarkConfig;

#[derive(Default)]
pub struct BasicMachine<F: PrimeField32 + TwoAdicField> {
    // Core instructions
    load32: Load32Instruction,
    store32: Store32Instruction,
    jal: JalInstruction,
    jalv: JalvInstruction,
    beq: BeqInstruction,
    bne: BneInstruction,
    imm32: Imm32Instruction,
    stop: StopInstruction,
    loadfp: LoadFpInstruction,

    // ALU instructions
    add32: Add32Instruction,
    sub32: Sub32Instruction,
    mul32: Mul32Instruction,
    mulhs32: Mulhs32Instruction,
    mulhu32: Mulhu32Instruction,
    div32: Div32Instruction,
    sdiv32: SDiv32Instruction,
    shl32: Shl32Instruction,
    shr32: Shr32Instruction,
    sra32: Sra32Instruction,
    lt32: Lt32Instruction,
    lte32: Lte32Instruction,
    and32: And32Instruction,
    or32: Or32Instruction,
    xor32: Xor32Instruction,
    ne32: Ne32Instruction,
    eq32: Eq32Instruction,

    // Input/output instructions
    read: ReadAdviceInstruction,
    write: WriteInstruction,

    // Chips
    cpu: CpuChip,
    program: ProgramChip,
    mem: MemoryChip,
    add_u32: Add32Chip,
    sub_u32: Sub32Chip,
    mul_u32: Mul32Chip,
    div_u32: Div32Chip,
    shift_u32: Shift32Chip,
    lt_u32: Lt32Chip,
    com_u32: Com32Chip,
    bitwise_u32: Bitwise32Chip,
    output: OutputChip,
    range: RangeCheckerChip<256>,
    static_data: StaticDataChip,

    _phantom_sc: PhantomData<fn() -> F>,
}

impl<F: PrimeField32 + TwoAdicField> Machine<F> for BasicMachine<F> {
    fn run<Adv>(&mut self, program: &ProgramROM<i32>, advice: &mut Adv)
    where
        Adv: AdviceProvider
    {
        self.initialize_memory();

        loop {
            // Fetch
            let pc = self.cpu().pc;
            let instruction = program.get_instruction(pc);
            let opcode = instruction.opcode;
            let ops = instruction.operands;

            // Execute
            match opcode {
                <Load32Instruction as Instruction<Self, F>>::OPCODE =>
                    Load32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Store32Instruction as Instruction<Self, F>>::OPCODE =>
                    Store32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <JalInstruction as Instruction<Self, F>>::OPCODE =>
                    JalInstruction::execute_with_advice::<Adv>(self, ops, advice),
                <JalvInstruction as Instruction<Self, F>>::OPCODE =>
                    JalvInstruction::execute_with_advice::<Adv>(self, ops, advice),
                <BeqInstruction as Instruction<Self, F>>::OPCODE =>
                    BeqInstruction::execute_with_advice::<Adv>(self, ops, advice),
                <BneInstruction as Instruction<Self, F>>::OPCODE =>
                    BneInstruction::execute_with_advice::<Adv>(self, ops, advice),
                <Imm32Instruction as Instruction<Self, F>>::OPCODE =>
                    Imm32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <StopInstruction as Instruction<Self, F>>::OPCODE =>
                    StopInstruction::execute_with_advice::<Adv>(self, ops, advice),
                <LoadFpInstruction as Instruction<Self, F>>::OPCODE =>
                    LoadFpInstruction::execute_with_advice::<Adv>(self, ops, advice),
                <Sub32Instruction as Instruction<Self, F>>::OPCODE =>
                    Sub32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Mul32Instruction as Instruction<Self, F>>::OPCODE =>
                    Mul32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Mulhs32Instruction as Instruction<Self, F>>::OPCODE =>
                    Mulhs32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Mulhu32Instruction as Instruction<Self, F>>::OPCODE =>
                    Mulhu32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Div32Instruction as Instruction<Self, F>>::OPCODE =>
                    Div32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <SDiv32Instruction as Instruction<Self, F>>::OPCODE =>
                    SDiv32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Shl32Instruction as Instruction<Self, F>>::OPCODE =>
                    Shl32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Shr32Instruction as Instruction<Self, F>>::OPCODE =>
                    Shr32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Lt32Instruction as Instruction<Self, F>>::OPCODE =>
                    Lt32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Lte32Instruction as Instruction<Self, F>>::OPCODE =>
                    Lte32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <And32Instruction as Instruction<Self, F>>::OPCODE =>
                    And32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Or32Instruction as Instruction<Self, F>>::OPCODE =>
                    Or32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Xor32Instruction as Instruction<Self, F>>::OPCODE =>
                    Xor32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Ne32Instruction as Instruction<Self, F>>::OPCODE =>
                    Ne32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <Eq32Instruction as Instruction<Self, F>>::OPCODE =>
                    Eq32Instruction::execute_with_advice::<Adv>(self, ops, advice),
                <ReadAdviceInstruction as Instruction<Self, F>>::OPCODE =>
                    ReadAdviceInstruction::execute_with_advice::<Adv>(self, ops, advice),
                <WriteInstruction as Instruction<Self, F>>::OPCODE =>
                    WriteInstruction::execute_with_advice::<Adv>(self, ops, advice),
                _ => panic!("Unrecognized opcode: {}", opcode),
            };
            self.read_word(pc as usize);

            // A STOP instruction signals the end of the program
            if opcode == <StopInstruction as Instruction<Self, F>>::OPCODE {
                break;
            }
        }

        // Record padded STOP instructions
        let n = self.cpu().clock.next_power_of_two() - self.cpu().clock;
        for _ in 0..n {
            self.read_word(self.cpu().pc as usize);
        }
    }

    fn prove<SC>(&self, config: &SC) -> MachineProof<SC>
    where
        SC: StarkConfig<Val = F>
    {
        todo!()
    }

    fn verify<SC>(&self, config: &SC, proof: &MachineProof<SC>) -> Result<(), ()>
    where
        SC: StarkConfig<Val = F>
    {
        todo!()
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithGeneralBus<F> for BasicMachine<F> {
    fn general_bus(&self) -> BusArgument {
        BusArgument::Global(0)
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithProgramBus<F> for BasicMachine<F> {
    fn program_bus(&self) -> BusArgument {
        BusArgument::Global(1)
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithMemBus<F> for BasicMachine<F> {
    fn mem_bus(&self) -> BusArgument {
        BusArgument::Global(2)
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithRangeBus8<F> for BasicMachine<F> {
    fn range_bus(&self) -> BusArgument {
        BusArgument::Global(3)
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithCpuChip<F> for BasicMachine<F> {
    fn cpu(&self) -> &CpuChip {
        &self.cpu
    }

    fn cpu_mut(&mut self) -> &mut CpuChip {
        &mut self.cpu
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithProgramChip<F> for BasicMachine<F> {
    fn program(&self) -> &ProgramChip {
        &self.program
    }

    fn program_mut(&mut self) -> &mut ProgramChip {
        &mut self.program
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithMemoryChip<F> for BasicMachine<F> {
    fn mem(&self) -> &MemoryChip {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut MemoryChip {
        &mut self.mem
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithAdd32Chip<F> for BasicMachine<F> {
    fn add_u32(&self) -> &Add32Chip {
        &self.add_u32
    }

    fn add_u32_mut(&mut self) -> &mut Add32Chip {
        &mut self.add_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithSub32Chip<F> for BasicMachine<F> {
    fn sub_u32(&self) -> &Sub32Chip {
        &self.sub_u32
    }

    fn sub_u32_mut(&mut self) -> &mut Sub32Chip {
        &mut self.sub_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithMul32Chip<F> for BasicMachine<F> {
    fn mul_u32(&self) -> &Mul32Chip {
        &self.mul_u32
    }

    fn mul_u32_mut(&mut self) -> &mut Mul32Chip {
        &mut self.mul_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithDiv32Chip<F> for BasicMachine<F> {
    fn div_u32(&self) -> &Div32Chip {
        &self.div_u32
    }

    fn div_u32_mut(&mut self) -> &mut Div32Chip {
        &mut self.div_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithBitwise32Chip<F> for BasicMachine<F> {
    fn bitwise_u32(&self) -> &Bitwise32Chip {
        &self.bitwise_u32
    }

    fn bitwise_u32_mut(&mut self) -> &mut Bitwise32Chip {
        &mut self.bitwise_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithLt32Chip<F> for BasicMachine<F> {
    fn lt_u32(&self) -> &Lt32Chip {
        &self.lt_u32
    }

    fn lt_u32_mut(&mut self) -> &mut Lt32Chip {
        &mut self.lt_u32
    }
}
impl<F: PrimeField32 + TwoAdicField> MachineWithCom32Chip<F> for BasicMachine<F> {
    fn com_u32(&self) -> &Com32Chip {
        &self.com_u32
    }

    fn com_u32_mut(&mut self) -> &mut Com32Chip {
        &mut self.com_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithShift32Chip<F> for BasicMachine<F> {
    fn shift_u32(&self) -> &Shift32Chip {
        &self.shift_u32
    }

    fn shift_u32_mut(&mut self) -> &mut Shift32Chip {
        &mut self.shift_u32
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithOutputChip<F> for BasicMachine<F> {
    fn output(&self) -> &OutputChip {
        &self.output
    }

    fn output_mut(&mut self) -> &mut OutputChip {
        &mut self.output
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithRangeChip<F, 256> for BasicMachine<F> {
    fn range(&self) -> &RangeCheckerChip<256> {
        &self.range
    }

    fn range_mut(&mut self) -> &mut RangeCheckerChip<256> {
        &mut self.range
    }
}

impl<F: PrimeField32 + TwoAdicField> MachineWithStaticDataChip<F> for BasicMachine<F> {
    fn static_data(&self) -> &StaticDataChip {
        &self.static_data
    }

    fn static_data_mut(&mut self) -> &mut StaticDataChip {
        &mut self.static_data
    }
}
