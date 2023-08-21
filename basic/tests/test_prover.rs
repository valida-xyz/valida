use p3_baby_bear::BabyBear;
use valida_alu_u32::add::{Add32Instruction, MachineWithAdd32Chip};
use valida_basic::BasicMachine;
use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    MachineWithCpuChip, StopInstruction, Store32Instruction,
};
use valida_machine::config::StarkConfigImpl;
use valida_machine::{Instruction, InstructionWord, Machine, Operands, ProgramROM, Word};
use valida_memory::MachineWithMemoryChip;

use p3_challenger::DuplexChallenger;
use p3_dft::Radix2BowersFft;
use p3_fri::{FriBasedPcs, FriConfigImpl, FriLdt};
use p3_ldt::QuotientMmcs;
use p3_mds::babybear::MDSMatrixBabyBear;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_poseidon::Poseidon;
use p3_symmetric::compression::TruncatedPermutation;
use p3_symmetric::sponge::PaddingFreeSponge;
use rand::thread_rng;

#[test]
fn prove_fibonacci() {
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
            operands: Operands([-4, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([-8, 0, 0, 0, 25]),
        },
        InstructionWord {
            opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([0, -16, -8, 0, 0]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([-20, 0, 0, 0, 28]),
        },
        InstructionWord {
            opcode: <JalInstruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([-28, fib_bb0, -28, 0, 0]),
        },
        InstructionWord {
            opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([0, -12, -24, 0, 0]),
        },
        InstructionWord {
            opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([0, 4, -12, 0, 0]),
        },
        InstructionWord {
            opcode: <StopInstruction as Instruction<BasicMachine>>::OPCODE,
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
            operands: Operands([0, -4, 12, 0, 0]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([-8, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([-12, 0, 0, 0, 1]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([-16, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: <BeqInstruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([fib_bb0_1, 0, 0, 0, 0]),
        },
    ]);

    //.LBB0_1:
    //	bne	.LBB0_2, -16(fp), -4(fp)
    //	beq	.LBB0_4, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: <BneInstruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([fib_bb0_2, -16, -4, 0, 0]),
        },
        InstructionWord {
            opcode: <BeqInstruction as Instruction<BasicMachine>>::OPCODE,
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
            opcode: <Add32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([-20, -8, -12, 0, 0]),
        },
        InstructionWord {
            opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([0, -8, -12, 0, 0]),
        },
        InstructionWord {
            opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([0, -12, -20, 0, 0]),
        },
        InstructionWord {
            opcode: <BeqInstruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([fib_bb0_3, 0, 0, 0, 0]),
        },
    ]);

    //; %bb.3:
    //	addi	-16(fp), -16(fp), 1
    //	beq	.LBB0_1, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([-16, -16, 1, 0, 1]),
        },
        InstructionWord {
            opcode: <BeqInstruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([fib_bb0_1, 0, 0, 0, 0]),
        },
    ]);

    //.LBB0_4:
    //	sw	4(fp), -8(fp)
    //	jalv	-4(fp), 0(fp), 8(fp)
    program.extend([
        InstructionWord {
            opcode: <Store32Instruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([0, 4, -8, 0, 0]),
        },
        InstructionWord {
            opcode: <JalvInstruction as Instruction<BasicMachine>>::OPCODE,
            operands: Operands([-4, 0, 8, 0, 0]),
        },
    ]);

    let mut machine = BasicMachine::default();
    let rom = ProgramROM::new(program);
    machine.cpu_mut().fp = 0x1000;
    machine.cpu_mut().save_register_state(); // TODO: Initial register state should be saved
                                             // automatically by the machine, not manually here
    machine.run(rom);

    type Val = BabyBear;
    type Dom = BabyBear;
    type Challenge = Val; // TODO
    type PackedChallenge = Challenge; // TODO

    type MyMds = MDSMatrixBabyBear;
    let mds = MyMds {};

    type Perm = Poseidon<Val, MyMds, 8, 7>;
    let perm = Perm::new_from_rng(5, 5, mds, &mut thread_rng()); // TODO: Use deterministic RNG

    type H4 = PaddingFreeSponge<Val, Perm, { 4 + 4 }>;
    let h4 = H4::new(perm.clone());

    type C = TruncatedPermutation<Val, Perm, 2, 4, { 2 * 4 }>;
    let c = C::new(perm.clone());

    type MyMmcs = MerkleTreeMmcs<Val, [Val; 4], H4, C>;
    let mmcs = MyMmcs::new(h4, c);

    type MyDft = Radix2BowersFft;
    let dft = MyDft::default();

    type Chal = DuplexChallenger<Val, Perm, 8>;
    type Quotient = QuotientMmcs<Dom, MyMmcs>;
    type MyFriConfig = FriConfigImpl<Val, Dom, Challenge, Quotient, MyMmcs, Chal>;
    let fri_config = MyFriConfig::new(40, mmcs.clone());
    let ldt = FriLdt { config: fri_config };

    type PCS = FriBasedPcs<MyFriConfig, MyMmcs, MyDft, Chal>;
    type MyConfig = StarkConfigImpl<Val, Challenge, PackedChallenge, PCS, MyDft, Chal>;

    let pcs = PCS::new(dft.clone(), 1, mmcs, ldt);
    let challenger = DuplexChallenger::new(perm);
    let config = MyConfig::new(pcs, dft, challenger);
    machine.prove(&config);

    assert_eq!(machine.cpu().clock, 192);
    assert_eq!(machine.cpu().operations.len(), 192);
    assert_eq!(machine.mem().operations.values().flatten().count(), 401);
    assert_eq!(machine.add_u32().operations.len(), 50);

    assert_eq!(
        *machine.mem().cells.get(&(0x1000 + 4)).unwrap(), // Return value
        Word([0, 1, 37, 17,])                             // 25th fibonacci number (75025)
    );
}
