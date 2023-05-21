#![no_std]

extern crate alloc;

use valida_alu_u32::{
    add::{Add32Chip, Add32Instruction, MachineWithAdd32Chip},
    //div::{Div32Instruction, MachineWithDiv32Chip},
    //lt::{Lt32Instruction, MachineWithLt32Chip},
    mul::{MachineWithMul32Chip, Mul32Chip, Mul32Instruction},
    //sub::{MachineWithSub32Chip, Sub32Chip, Sub32Instruction},
};
use valida_bus::{CpuMemBus, MachineWithGeneralBus, MachineWithMemBus, SharedCoprocessorBus};
use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    Load32Instruction, Store32Instruction,
};
use valida_cpu::{CpuChip, MachineWithCpuChip};
use valida_derive::Machine;
use valida_machine::{Fp, Instruction, Machine, ProgramROM};
use valida_memory::{MachineWithMemoryChip, MemoryChip};

// TODO: Emit instruction members in the derive macro instead of manually including

#[derive(Machine, Default)]
pub struct BasicMachine {
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
    //#[instruction(sub_u32)]
    //sub32: Sub32Instruction,
    #[instruction(mul_u32)]
    mul32: Mul32Instruction,
    //#[instruction(div_u32)]
    //div32: Div32Instruction,
    //#[instruction(lt_u32)]
    //lt32: LtInstruction,
    #[chip]
    cpu: CpuChip<Fp>,
    #[chip]
    mem: MemoryChip<Fp>,
    #[chip]
    add_u32: Add32Chip<Fp>,
    #[chip]
    mul_u32: Mul32Chip<Fp>,
    //#[chip]
    //sub_u32: Sub32Chip<F>,
    //#[chip]
    //div_u32: Div32Chip<F>,
    //#[chip]
    //lt_u32: Lt32Chip<F>,
}

impl MachineWithGeneralBus for BasicMachine {
    fn general_bus(&self) -> usize {
        0
    }
}

impl MachineWithMemBus for BasicMachine {
    fn mem_bus(&self) -> usize {
        1
    }
}

impl MachineWithCpuChip for BasicMachine {
    fn cpu(&self) -> &CpuChip<Fp> {
        &self.cpu
    }

    fn cpu_mut(&mut self) -> &mut CpuChip<Fp> {
        &mut self.cpu
    }
}

impl MachineWithMemoryChip for BasicMachine {
    fn mem(&self) -> &MemoryChip<Fp> {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut MemoryChip<Fp> {
        &mut self.mem
    }
}

impl MachineWithAdd32Chip for BasicMachine {
    fn add_u32(&self) -> &Add32Chip<Fp> {
        &self.add_u32
    }

    fn add_u32_mut(&mut self) -> &mut Add32Chip<Fp> {
        &mut self.add_u32
    }
}

//impl MachineWithSub32Chip for BasicMachine {
//    fn sub_u32(&self) -> &Sub32Chip {
//        &self.add_u32
//    }
//
//    fn sub_u32_mut(&mut self) -> &mut Sub32Chip {
//        &mut self.add_u32
//    }
//}

impl MachineWithMul32Chip for BasicMachine {
    fn mul_u32(&self) -> &Mul32Chip<Fp> {
        &self.mul_u32
    }

    fn mul_u32_mut(&mut self) -> &mut Mul32Chip<Fp> {
        &mut self.mul_u32
    }
}

//impl MachineWithDiv32Chip for BasicMachine {
//    fn div_u32(&self) -> &Div32Chip {
//        &self.div_u32
//    }
//
//    fn div_u32_mut(&mut self) -> &mut Div32Chip {
//        &mut self.div_u32
//    }
//}

//impl MachineWithLt32Chip for BasicMachine {
//    fn lt_u32(&self) -> &Lt32Chip {
//        &self.lt_u32
//    }
//
//    fn lt_u32_mut(&mut self) -> &mut Lt32Chip {
//        &mut self.lt_u32
//    }
//}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use p3_field::PrimeField;
    use p3_mersenne_31::Mersenne31 as Fp;
    use valida_machine::Operands;
    use valida_machine::{InstructionWord, Word};

    #[test]
    fn fibonacci() {
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
                opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-4, 0, 0, 0, 0]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-8, 0, 0, 0, 25]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[0, -16, -8, 0, 0]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-20, 0, 0, 0, 28]),
            },
            InstructionWord {
                opcode: <JalInstruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-28, fib_bb0, -28, 0, 0]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[0, -12, -24, 0, 0]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[0, 4, -12, 0, 0]),
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
                opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[0, -4, 12, 0, 0]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-8, 0, 0, 0, 0]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-12, 0, 0, 0, 1]),
            },
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-16, 0, 0, 0, 0]),
            },
            InstructionWord {
                opcode: <BeqInstruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[fib_bb0_1, 0, 0, 0, 0]),
            },
        ]);

        //.LBB0_1:
        //	bne	.LBB0_2, -16(fp), -4(fp)
        //	beq	.LBB0_4, 0(fp), 0(fp)
        program.extend([
            InstructionWord {
                opcode: <BneInstruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[fib_bb0_2, -16, -4, 0, 0]),
            },
            InstructionWord {
                opcode: <BeqInstruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[fib_bb0_4, 0, 0, 0, 0]),
            },
        ]);

        //; %bb.2:
        //	add	-20(fp), -8(fp), -12(fp)
        //	sw	-8(fp), -12(fp)
        //	sw	-12(fp), -20(fp)
        //	beq	.LBB0_3, 0(fp), 0(fp)
        program.extend([
            InstructionWord {
                opcode: <Add32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-20, -8, -12, 0, 0]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[0, -8, -12, 0, 0]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[0, -12, -20, 0, 0]),
            },
            InstructionWord {
                opcode: <BeqInstruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[fib_bb0_3, 0, 0, 0, 0]),
            },
        ]);

        //; %bb.3:
        //	addi	-16(fp), -16(fp), 1
        //	beq	.LBB0_1, 0(fp), 0(fp)
        program.extend([
            InstructionWord {
                opcode: <Add32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-16, -16, 1, 0, 1]),
            },
            InstructionWord {
                opcode: <BeqInstruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[fib_bb0_1, 0, 0, 0, 0]),
            },
        ]);

        //.LBB0_4:
        //	sw	4(fp), -8(fp)
        //	jalv	-4(fp), 0(fp), 8(fp)
        program.extend([
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[0, 4, -8, 0, 0]),
            },
            InstructionWord {
                opcode: <JalvInstruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-4, 0, 8, 0, 0]),
            },
        ]);

        let mut machine = BasicMachine::default();
        let rom = ProgramROM::new(program);
        machine.run(rom);

        assert_eq!(machine.cpu().clock, Fp::from_canonical_usize(191));
        assert_eq!(machine.cpu().operations.len(), 141);
        assert_eq!(machine.mem().operations.len(), 191);
        assert_eq!(machine.add_u32().operations.len(), 50);

        assert_eq!(
            *machine.mem().cells.get(&Fp::from_canonical_u32(4)).unwrap(), // Return value
            Word::from([
                Fp::from_canonical_u8(0),
                Fp::from_canonical_u8(1),
                Fp::from_canonical_u8(37),
                Fp::from_canonical_u8(17)
            ])  // 25th fibonacci number (75025)
        );
    }

    #[test]
    fn store32() {
        let program = vec![
            InstructionWord {
                opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::<Fp>::from_i32_slice(&[-4, 0, 0, 0, 42]),
            },
            InstructionWord {
                opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
                operands: Operands::from_i32_slice(&[0, -8, -4, 0, 0]),
            },
            InstructionWord {
                opcode: 0,
                operands: Operands::default(),
            },
        ];

        let mut machine = BasicMachine::default();
        let rom = ProgramROM::new(program);
        machine.run(rom);

        assert_eq!(machine.cpu().pc, Fp::from_canonical_u32(2));
        assert_eq!(machine.cpu().fp, Fp::from_canonical_u32(0));
        assert_eq!(machine.cpu().clock, Fp::from_canonical_u32(2));
        assert_eq!(
            *machine
                .mem()
                .cells
                .get(&-Fp::from_canonical_u32(4))
                .unwrap(),
            Word::from(Fp::from_canonical_u32(42))
        );
        assert_eq!(
            *machine
                .mem()
                .cells
                .get(&-Fp::from_canonical_u32(8))
                .unwrap(),
            Word::from(Fp::from_canonical_u32(42))
        );
    }
}
