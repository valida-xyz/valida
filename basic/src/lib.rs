#![no_std]
#![allow(unused)]

extern crate alloc;

use core::marker::PhantomData;
use p3_field::{ExtensionField, PrimeField64};
use valida_alu_u32::{
    add::{Add32Chip, Add32Instruction, MachineWithAdd32Chip},
    mul::{MachineWithMul32Chip, Mul32Chip, Mul32Instruction},
};
use valida_bus::{MachineWithGeneralBus, MachineWithMemBus, MachineWithRangeBus8};
use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    Load32Instruction, Store32Instruction,
};
use valida_cpu::{CpuChip, MachineWithCpuChip};
use valida_derive::Machine;
use valida_machine::{
    AbstractExtensionField, AbstractField, BusArgument, Chip, Instruction, Machine, ProgramROM,
};
use valida_memory::{MachineWithMemoryChip, MemoryChip};
use valida_program::MachineWithProgramChip;
use valida_range::{MachineWithRangeChip, RangeCheckerChip};

use p3_maybe_rayon::*;
use valida_program::ProgramChip;

#[derive(Machine, Default)]
#[machine_fields(F, EF)]
pub struct BasicMachine<F: PrimeField64, EF: ExtensionField<F>> {
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

    // ALU instructions
    #[instruction(add_u32)]
    add32: Add32Instruction,
    #[instruction(mul_u32)]
    mul32: Mul32Instruction,

    #[chip]
    cpu: CpuChip,
    #[chip]
    program: ProgramChip<F>,
    #[chip]
    mem: MemoryChip,
    #[chip]
    add_u32: Add32Chip,
    #[chip]
    mul_u32: Mul32Chip,
    #[chip]
    range: RangeCheckerChip, // TODO: Specify 8-bit RC chip

    _phantom_ef: PhantomData<EF>,
}

impl<F: PrimeField64, EF: ExtensionField<F>> MachineWithGeneralBus for BasicMachine<F, EF> {
    fn general_bus(&self) -> BusArgument {
        BusArgument::Global(0)
    }
}

impl<F: PrimeField64, EF: ExtensionField<F>> MachineWithMemBus for BasicMachine<F, EF> {
    fn mem_bus(&self) -> BusArgument {
        BusArgument::Global(1)
    }
}

impl<F: PrimeField64, EF: ExtensionField<F>> MachineWithRangeBus8 for BasicMachine<F, EF> {
    fn range_bus(&self) -> BusArgument {
        BusArgument::Global(2)
    }
}

impl<F: PrimeField64, EF: ExtensionField<F>> MachineWithCpuChip for BasicMachine<F, EF> {
    fn cpu(&self) -> &CpuChip {
        &self.cpu
    }

    fn cpu_mut(&mut self) -> &mut CpuChip {
        &mut self.cpu
    }
}

impl<F: PrimeField64, EF: ExtensionField<F>> MachineWithProgramChip for BasicMachine<F, EF> {
    fn program(&self) -> &ProgramChip<Self::F> {
        &self.program
    }

    fn program_mut(&mut self) -> &mut ProgramChip<Self::F> {
        &mut self.program
    }
}

impl<F: PrimeField64, EF: ExtensionField<F>> MachineWithMemoryChip for BasicMachine<F, EF> {
    fn mem(&self) -> &MemoryChip {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut MemoryChip {
        &mut self.mem
    }
}

impl<F: PrimeField64, EF: ExtensionField<F>> MachineWithAdd32Chip for BasicMachine<F, EF> {
    fn add_u32(&self) -> &Add32Chip {
        &self.add_u32
    }

    fn add_u32_mut(&mut self) -> &mut Add32Chip {
        &mut self.add_u32
    }
}

impl<F: PrimeField64, EF: ExtensionField<F>> MachineWithMul32Chip for BasicMachine<F, EF> {
    fn mul_u32(&self) -> &Mul32Chip {
        &self.mul_u32
    }

    fn mul_u32_mut(&mut self) -> &mut Mul32Chip {
        &mut self.mul_u32
    }
}

impl<F: PrimeField64, EF: ExtensionField<F>> MachineWithRangeChip for BasicMachine<F, EF> {
    fn range(&self) -> &RangeCheckerChip {
        &self.range
    }

    fn range_mut(&mut self) -> &mut RangeCheckerChip {
        &mut self.range
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use p3_challenger::DuplexChallenger;
    use p3_merkle_tree::MerkleTreeMMCS;
    use p3_mersenne_31::Mersenne31;
    use p3_poseidon::Poseidon;
    use p3_symmetric::compression::TruncatedPermutation;
    use p3_symmetric::mds::NaiveMDSMatrix;
    use p3_symmetric::sponge::PaddingFreeSponge;
    use p3_tensor_pcs::TensorPCS;
    use rand::thread_rng;
    use valida_machine::config::StarkConfigImpl;
    use valida_machine::Operands;
    use valida_machine::{InstructionWord, Word};

    #[test]
    fn fibonacci() {
        type Val = Mersenne31;
        type Challenge = Val; // TODO
        type PackedChallenge = Challenge; // TODO

        let mut program = vec![];

        // Label locations
        let fib_bb0 = 8;
        let fib_bb0_1 = 13;
        let fib_bb0_2 = 15;
        let fib_bb0_3 = 19;
        let fib_bb0_4 = 21;

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
                opcode: <Imm32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-4, 0, 0, 0, 0]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-8, 0, 0, 0, 25]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([0, -16, -8, 0, 0]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-20, 0, 0, 0, 28]),
            },
            InstructionWord {
                opcode: <JalInstruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-28, fib_bb0, -28, 0, 0]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([0, -12, -24, 0, 0]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([0, 4, -12, 0, 0]),
            },
            InstructionWord {
                opcode: 0,
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
                opcode: <Store32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([0, -4, 12, 0, 0]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-8, 0, 0, 0, 0]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-12, 0, 0, 0, 1]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-16, 0, 0, 0, 0]),
            },
            InstructionWord {
                opcode: <BeqInstruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([fib_bb0_1, 0, 0, 0, 0]),
            },
        ]);

        //.LBB0_1:
        //	bne	.LBB0_2, -16(fp), -4(fp)
        //	beq	.LBB0_4, 0(fp), 0(fp)
        program.extend([
            InstructionWord {
                opcode: <BneInstruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([fib_bb0_2, -16, -4, 0, 0]),
            },
            InstructionWord {
                opcode: <BeqInstruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
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
                opcode: <Add32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-20, -8, -12, 0, 0]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([0, -8, -12, 0, 0]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([0, -12, -20, 0, 0]),
            },
            InstructionWord {
                opcode: <BeqInstruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([fib_bb0_3, 0, 0, 0, 0]),
            },
        ]);

        //; %bb.3:
        //	addi	-16(fp), -16(fp), 1
        //	beq	.LBB0_1, 0(fp), 0(fp)
        program.extend([
            InstructionWord {
                opcode: <Add32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-16, -16, 1, 0, 1]),
            },
            InstructionWord {
                opcode: <BeqInstruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([fib_bb0_1, 0, 0, 0, 0]),
            },
        ]);

        //.LBB0_4:
        //	sw	4(fp), -8(fp)
        //	jalv	-4(fp), 0(fp), 8(fp)
        program.extend([
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([0, 4, -8, 0, 0]),
            },
            InstructionWord {
                opcode: <JalvInstruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-4, 0, 8, 0, 0]),
            },
        ]);

        let mut machine = BasicMachine::default();
        let rom = ProgramROM::new(program);
        machine.cpu_mut().fp = 0x1000;
        machine.cpu_mut().save_register_state(); // TODO: Initial register state should be saved
                                                 // automatically by the machine, not manually here
        machine.run(rom);

        let mds = NaiveMDSMatrix::<Val, 8>::new([[Val::ONE; 8]; 8]); // TODO: Use a real MDS matrix
        type Perm = Poseidon<Val, NaiveMDSMatrix<Val, 8>, 8, 7>;
        let perm = Perm::new_from_rng(5, 5, mds, &mut thread_rng()); // TODO: Use deterministic RNG
        let h4 = PaddingFreeSponge::<Val, Perm, { 4 + 4 }>::new(perm.clone());
        let c = TruncatedPermutation::<Val, Perm, 2, 4, { 2 * 4 }>::new(perm.clone());
        let mmcs = MerkleTreeMMCS::new(h4, c);
        let codes = p3_brakedown::fast_registry();
        let pcs = TensorPCS::new(codes, mmcs);
        let challenger = DuplexChallenger::new(perm);
        let config = StarkConfigImpl::<Val, Challenge, PackedChallenge, _, _>::new(pcs, challenger);
        machine.prove(&config);

        assert_eq!(machine.cpu().clock, 191);
        assert_eq!(machine.cpu().operations.len(), 191);
        assert_eq!(machine.mem().operations.values().flatten().count(), 401);
        assert_eq!(machine.add_u32().operations.len(), 50);

        assert_eq!(
            *machine.mem().cells.get(&(0x1000 + 4)).unwrap(), // Return value
            Word([0, 1, 37, 17,])                             // 25th fibonacci number (75025)
        );
    }

    #[test]
    fn store32() {
        type Val = Mersenne31;
        type Challenge = Val; // TODO

        let program = vec![
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([-4, 0, 0, 0, 42]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine<Val, Challenge>>>::OPCODE,
                operands: Operands([0, -8, -4, 0, 0]),
            },
            InstructionWord {
                opcode: 0,
                operands: Operands::default(),
            },
        ];

        let mut machine = BasicMachine::<Val, Challenge>::default();
        let rom = ProgramROM::new(program);
        machine.cpu_mut().fp = 0x1000;
        machine.run(rom);
        //machine.prove();

        assert_eq!(machine.cpu().pc, 2);
        assert_eq!(machine.cpu().fp, 0x1000);
        assert_eq!(machine.cpu().clock, 2);
        assert_eq!(
            *machine.mem().cells.get(&(0x1000 - 4)).unwrap(),
            Word([0, 0, 0, 42])
        );
        assert_eq!(
            *machine.mem().cells.get(&(0x1000 - 8)).unwrap(),
            Word([0, 0, 0, 42])
        );
    }
}
