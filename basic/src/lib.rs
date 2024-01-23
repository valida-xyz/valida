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
    div::{Div32Chip, Div32Instruction, MachineWithDiv32Chip, SDiv32Instruction},
    lt::{Lt32Chip, Lt32Instruction, MachineWithLt32Chip},
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
    Load32Instruction, ReadAdviceInstruction, StopInstruction, Store32Instruction,
};
use valida_cpu::{CpuChip, MachineWithCpuChip};
use valida_derive::Machine;
use valida_machine::{
    proof::{ChipProof, MachineProof},
    AbstractExtensionField, AbstractField, AdviceProvider, BusArgument, Chip, ExtensionField,
    Instruction, Machine, PrimeField64, ProgramROM, ValidaAirBuilder,
};
use valida_memory::{MachineWithMemoryChip, MemoryChip};
use valida_output::{MachineWithOutputChip, OutputChip, WriteInstruction};
use valida_program::{MachineWithProgramChip, ProgramChip};
use valida_range::{MachineWithRangeChip, RangeCheckerChip};

use p3_maybe_rayon::prelude::*;
use valida_machine::config::StarkConfig;

#[derive(Machine, Default)]
#[machine_fields(F)]
pub struct BasicMachine<F: PrimeField32 + TwoAdicField> {
    // Core instructions
    #[instruction]
    load32: Load32Instruction,

    #[instruction]
    store32: Store32Instruction,

    #[instruction]
    jal: JalInstruction,

    #[instruction]
    jalv: JalvInstruction,

    #[instruction]
    beq: BeqInstruction,

    #[instruction]
    bne: BneInstruction,

    #[instruction]
    imm32: Imm32Instruction,

    #[instruction]
    stop: StopInstruction,

    // ALU instructions
    #[instruction(add_u32)]
    add32: Add32Instruction,

    #[instruction(sub_u32)]
    sub32: Sub32Instruction,

    #[instruction(mul_u32)]
    mul32: Mul32Instruction,

    #[instruction(mul_u32)]
    mulhs32: Mulhs32Instruction,

    #[instruction(mul_u32)]
    mulhu32: Mulhu32Instruction,

    #[instruction(div_u32)]
    div32: Div32Instruction,

    #[instruction(div_u32)]
    sdiv32: SDiv32Instruction,

    #[instruction(shift_u32)]
    shl32: Shl32Instruction,

    #[instruction(shift_u32)]
    shr32: Shr32Instruction,

    #[instruction(shift_u32)]
    sra32: Sra32Instruction,

    #[instruction(lt_u32)]
    lt32: Lt32Instruction,

    #[instruction(bitwise_u32)]
    and32: And32Instruction,

    #[instruction(bitwise_u32)]
    or32: Or32Instruction,

    #[instruction(bitwise_u32)]
    xor32: Xor32Instruction,

    // Input/output instructions
    #[instruction]
    read: ReadAdviceInstruction,

    #[instruction(output)]
    write: WriteInstruction,

    #[chip]
    cpu: CpuChip,

    #[chip]
    program: ProgramChip,

    #[chip]
    mem: MemoryChip,

    #[chip]
    add_u32: Add32Chip,

    #[chip]
    sub_u32: Sub32Chip,

    #[chip]
    mul_u32: Mul32Chip,

    #[chip]
    div_u32: Div32Chip,

    #[chip]
    shift_u32: Shift32Chip,

    #[chip]
    lt_u32: Lt32Chip,

    #[chip]
    bitwise_u32: Bitwise32Chip,

    #[chip]
    output: OutputChip,

    #[chip]
    range: RangeCheckerChip<256>,

    _phantom_sc: PhantomData<fn() -> F>,
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

// impl<F: PrimeField32 + TwoAdicField> Machine<F> for BasicMachine<F> {
//     fn run<Adv: AdviceProvider>(&mut self, program: &ProgramROM<i32>, advice: &mut Adv) {
//         loop {
//             let pc = self.cpu.pc;
//             let instruction = program.get_instruction(pc);
//             let opcode = instruction.opcode;
//             let ops = instruction.operands;
//             match opcode {
//                 <Load32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Load32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Store32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Store32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <JalInstruction as Instruction<Self, F>>::OPCODE => {
//                     JalInstruction::execute_with_advice(self, ops, advice);
//                 }
//                 <JalvInstruction as Instruction<Self, F>>::OPCODE => {
//                     JalvInstruction::execute_with_advice(self, ops, advice);
//                 }
//                 <BeqInstruction as Instruction<Self, F>>::OPCODE => {
//                     BeqInstruction::execute_with_advice(self, ops, advice);
//                 }
//                 <BneInstruction as Instruction<Self, F>>::OPCODE => {
//                     BneInstruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Imm32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Imm32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Add32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Add32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Sub32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Sub32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Mul32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Mul32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Mulhs32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Mulhs32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Mulhu32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Mulhu32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Div32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Div32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <SDiv32Instruction as Instruction<Self, F>>::OPCODE => {
//                     SDiv32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Shl32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Shl32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Shr32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Shr32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Sra32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Sra32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Lt32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Lt32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <And32Instruction as Instruction<Self, F>>::OPCODE => {
//                     And32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Or32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Or32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <Xor32Instruction as Instruction<Self, F>>::OPCODE => {
//                     Xor32Instruction::execute_with_advice(self, ops, advice);
//                 }
//                 <ReadAdviceInstruction as Instruction<Self, F>>::OPCODE => {
//                     ReadAdviceInstruction::execute_with_advice(self, ops, advice);
//                 }
//                 <WriteInstruction as Instruction<Self, F>>::OPCODE => {
//                     WriteInstruction::execute_with_advice(self, ops, advice);
//                 }
//                 <StopInstruction as Instruction<Self, F>>::OPCODE => {
//                     StopInstruction::execute_with_advice(self, ops, advice);
//                 }
//                 _ => {}
//             }
//
//             self.read_word(pc as usize);
//
//             if opcode == <StopInstruction as Instruction<Self, F>>::OPCODE {
//                 break;
//             }
//         }
//         let n = self.cpu.clock.next_power_of_two() - self.cpu.clock;
//         for _ in 0..n {
//             self.read_word(self.cpu.pc as usize);
//         }
//     }
//
//     fn add_chip_trace<SC, A>(
//         &self,
//         config: &SC,
//         challenger: &mut SC::Challenger,
//         trace_commitments: &mut Vec<ProverData<SC>>,
//         quotient_commitments: &mut Vec<ProverData<SC>>,
//         log_degrees: &mut Vec<usize>,
//         log_quotient_degrees: &mut Vec<usize>,
//         chip: &A,
//         trace: RowMajorMatrix<<SC as StarkConfig>::Val>,
//     ) where
//         SC: StarkConfig,
//         A: Air<SymbolicAirBuilder<SC::Val>> + for<'a> Air<ProverConstraintFolder<'a, SC>>,
//     {
//         let (trace_lde, quotient_lde, log_degree, log_quotient_degree) =
//             get_trace_and_quotient_ldes(config, trace, chip, challenger);
//         trace_commitments.push(trace_lde);
//         quotient_commitments.push(quotient_lde);
//         log_degrees.push(log_degree);
//         log_quotient_degrees.push(log_quotient_degree);
//     }
//
//     fn prove<SC>(&self, config: &SC, challenger: &mut SC::Challenger) -> MachineProof<SC>
//     where
//         SC: StarkConfig<Val = F>,
//     {
//         let mut trace_commitments = Vec::new();
//         let mut quotient_commitments = Vec::new();
//         let mut log_degrees = Vec::new();
//         let mut log_quotient_degrees = Vec::new();
//         /*
//                 let air = &self.cpu();
//                 assert_eq!(air.operations.len() > 0, true);
//                 let trace = air.generate_trace(air, self);
//                 self.add_chip_trace(
//                     config,
//                     challenger,
//                     &mut trace_commitments,
//                     &mut quotient_commitments,
//                     &mut log_degrees,
//                     &mut log_quotient_degrees,
//                     air,
//                     trace,
//                 );
//         */
//         if self.add_u32.operations.len() > 0 {
//             let air = &self.add_u32;
//             let trace = <Add32Chip as Chip<BasicMachine<F>, SC>>::generate_trace(air, self);
//
//             self.add_chip_trace(
//                 config,
//                 challenger,
//                 &mut trace_commitments,
//                 &mut quotient_commitments,
//                 &mut log_degrees,
//                 &mut log_quotient_degrees,
//                 air,
//                 trace,
//             );
//         }
//         if self.sub_u32.operations.len() > 0 {
//             let air = &self.sub_u32;
//             let trace = <Sub32Chip as Chip<BasicMachine<F>, SC>>::generate_trace(air, self);
//
//             self.add_chip_trace(
//                 config,
//                 challenger,
//                 &mut trace_commitments,
//                 &mut quotient_commitments,
//                 &mut log_degrees,
//                 &mut log_quotient_degrees,
//                 air,
//                 trace,
//             );
//         }
//         if self.mul_u32.operations.len() > 0 {
//             let air = &self.mul_u32;
//             let trace = <Mul32Chip as Chip<BasicMachine<F>, SC>>::generate_trace(air, self);
//
//             self.add_chip_trace(
//                 config,
//                 challenger,
//                 &mut trace_commitments,
//                 &mut quotient_commitments,
//                 &mut log_degrees,
//                 &mut log_quotient_degrees,
//                 air,
//                 trace,
//             );
//         }
//         if self.div_u32.operations.len() > 0 {
//             let air = &self.div_u32;
//
//             let trace = <Div32Chip as Chip<BasicMachine<F>, SC>>::generate_trace(air, self);
//             self.add_chip_trace(
//                 config,
//                 challenger,
//                 &mut trace_commitments,
//                 &mut quotient_commitments,
//                 &mut log_degrees,
//                 &mut log_quotient_degrees,
//                 air,
//                 trace,
//             );
//         }
//         if self.shift_u32.operations.len() > 0 {
//             let air = &self.shift_u32;
//             let trace = <Shift32Chip as Chip<BasicMachine<F>, SC>>::generate_trace(air, self);
//
//             self.add_chip_trace(
//                 config,
//                 challenger,
//                 &mut trace_commitments,
//                 &mut quotient_commitments,
//                 &mut log_degrees,
//                 &mut log_quotient_degrees,
//                 air,
//                 trace,
//             );
//         }
//         if self.lt_u32.operations.len() > 0 {
//             let air = &self.lt_u32;
//             let trace = <Lt32Chip as Chip<BasicMachine<F>, SC>>::generate_trace(air, self);
//
//             self.add_chip_trace(
//                 config,
//                 challenger,
//                 &mut trace_commitments,
//                 &mut quotient_commitments,
//                 &mut log_degrees,
//                 &mut log_quotient_degrees,
//                 air,
//                 trace,
//             );
//         }
//
//         if self.bitwise_u32.operations.len() > 0 {
//             let air = &self.bitwise_u32;
//             let trace = <Bitwise32Chip as Chip<BasicMachine<F>, SC>>::generate_trace(air, self);
//
//             self.add_chip_trace(
//                 config,
//                 challenger,
//                 &mut trace_commitments,
//                 &mut quotient_commitments,
//                 &mut log_degrees,
//                 &mut log_quotient_degrees,
//                 air,
//                 trace,
//             );
//         }
//
//         let pcs = config.pcs();
//         let (aggregated_commitment, aggregated_trace) = pcs.combine(&trace_commitments);
//         let (aggregated_quotient_commitment, aggregated_quotient_trace) =
//             pcs.combine(&quotient_commitments);
//         let max_log_degree = log_degrees.iter().max().unwrap();
//         let max_quotient_degree = log_quotient_degrees.iter().max().unwrap();
//         let (opening_proof, opened_values) = open(
//             config,
//             &aggregated_trace,
//             &aggregated_quotient_trace,
//             *max_log_degree,
//             *max_quotient_degree,
//             challenger,
//         );
//
//         let commitments = Commitments {
//             trace: aggregated_commitment,
//             quotient_chunks: aggregated_quotient_commitment,
//         };
//         MachineProof {
//             chip_proof: ChipProof {
//                 proof: Proof {
//                     commitments,
//                     opened_values,
//                     opening_proof,
//                     degree_bits: *max_log_degree,
//                 },
//             },
//             phantom: PhantomData::default(),
//         }
//     }
//
//     fn verify<SC>(proof: &MachineProof<SC>) -> Result<(), ()>
//     where
//         SC: StarkConfig<Val = F>,
//     {
//         Ok(())
//     }
// }
