extern crate core;

use p3_baby_bear::BabyBear;
use p3_fri::{TwoAdicFriPcs, TwoAdicFriPcsConfig};
use valida_alu_u32::add::{Add32Instruction, MachineWithAdd32Chip};
use valida_basic::BasicMachine;
use valida_cpu::{
    BeqInstruction, BneInstruction, Imm32Instruction, JalInstruction, JalvInstruction,
    MachineWithCpuChip, StopInstruction,
};
use valida_machine::{
    FixedAdviceProvider, Instruction, InstructionWord, Machine, MachineProof, Operands, ProgramROM,
    Word,
};

use valida_memory::MachineWithMemoryChip;
use valida_opcodes::BYTES_PER_INSTR;
use valida_program::MachineWithProgramChip;

use p3_challenger::DuplexChallenger;
use p3_dft::Radix2Bowers;
use p3_field::extension::BinomialExtensionField;
use p3_field::Field;
use p3_fri::FriConfig;
use p3_keccak::Keccak256Hash;
use p3_mds::coset_mds::CosetMds;
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_poseidon::Poseidon;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher32};
use rand::thread_rng;
use valida_machine::StarkConfigImpl;
use valida_machine::__internal::p3_commit::ExtensionMmcs;

fn main() {
    prove_fibonacci()
}

fn prove_fibonacci() {
    let mut program = vec![];

    // Label locations
    let bytes_per_instr = BYTES_PER_INSTR as i32;
    let fib_bb0 = 8 * bytes_per_instr;
    let fib_bb0_1 = 13 * bytes_per_instr;
    let fib_bb0_2 = 15 * bytes_per_instr;
    let fib_bb0_3 = 19 * bytes_per_instr;
    let fib_bb0_4 = 21 * bytes_per_instr;

    //main:                                   ; @main
    //; %bb.0:
    //	imm32	-4(fp), 0, 0, 0, 0
    //	imm32	-8(fp), 0, 0, 0, 100
    //	addi	-16(fp), -8(fp), 0
    //	imm32	-20(fp), 0, 0, 0, 28
    //	jal	-28(fp), fib, -28
    //	addi	-12(fp), -24(fp), 0
    //	addi	4(fp), -12(fp), 0
    //	exit
    //...
    program.extend([
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-4, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-8, 0, 0, 0, 100]),
        },
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-16, -8, 0, 0, 1]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-20, 0, 0, 0, 28]),
        },
        InstructionWord {
            opcode: <JalInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-28, fib_bb0, -28, 0, 0]),
        },
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-12, -24, 0, 0, 1]),
        },
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([4, -12, 0, 0, 1]),
        },
        InstructionWord {
            opcode: <StopInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands::default(),
        },
    ]);

    //fib:                                    ; @fib
    //; %bb.0:
    //	addi	-4(fp), 12(fp), 0
    //	imm32	-8(fp), 0, 0, 0, 0
    //	imm32	-12(fp), 0, 0, 0, 1
    //	imm32	-16(fp), 0, 0, 0, 0
    //	beq	.LBB0_1, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-4, 12, 0, 0, 1]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-8, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-12, 0, 0, 0, 1]),
        },
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-16, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: <BeqInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([fib_bb0_1, 0, 0, 0, 0]),
        },
    ]);

    //.LBB0_1:
    //	bne	.LBB0_2, -16(fp), -4(fp)
    //	beq	.LBB0_4, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: <BneInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([fib_bb0_2, -16, -4, 0, 0]),
        },
        InstructionWord {
            opcode: <BeqInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([fib_bb0_4, 0, 0, 0, 0]),
        },
    ]);

    //; %bb.2:
    //	add	-20(fp), -8(fp), -12(fp)
    //	addi	-8(fp), -12(fp), 0
    //	addi	-12(fp), -20(fp), 0
    //	beq	.LBB0_3, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-20, -8, -12, 0, 0]),
        },
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-8, -12, 0, 0, 1]),
        },
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-12, -20, 0, 0, 1]),
        },
        InstructionWord {
            opcode: <BeqInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([fib_bb0_3, 0, 0, 0, 0]),
        },
    ]);

    //; %bb.3:
    //	addi	-16(fp), -16(fp), 1
    //	beq	.LBB0_1, 0(fp), 0(fp)
    program.extend([
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-16, -16, 1, 0, 1]),
        },
        InstructionWord {
            opcode: <BeqInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([fib_bb0_1, 0, 0, 0, 0]),
        },
    ]);

    //.LBB0_4:
    //	addi	4(fp), -8(fp), 0
    //	jalv	-4(fp), 0(fp), 8(fp)
    program.extend([
        InstructionWord {
            opcode: <Add32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([4, -8, 0, 0, 1]),
        },
        InstructionWord {
            opcode: <JalvInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-4, 0, 8, 0, 0]),
        },
    ]);

    let mut machine = BasicMachine::<Val>::default();
    let rom = ProgramROM::new(program);
    machine.program_mut().set_program_rom(&rom);
    machine.cpu_mut().fp = 0x1000;
    machine.cpu_mut().save_register_state(); // TODO: Initial register state should be saved
                                             // automatically by the machine, not manually here
    machine.run(&rom, &mut FixedAdviceProvider::empty());

    type Val = BabyBear;
    type Challenge = BinomialExtensionField<Val, 5>;
    type PackedChallenge = BinomialExtensionField<<Val as Field>::Packing, 5>;

    type Mds16 = CosetMds<Val, 16>;
    let mds16 = Mds16::default();

    type Perm16 = Poseidon<Val, Mds16, 16, 5>;
    let perm16 = Perm16::new_from_rng(4, 22, mds16, &mut thread_rng()); // TODO: Use deterministic RNG

    type MyHash = SerializingHasher32<Keccak256Hash>;
    let hash = MyHash::new(Keccak256Hash {});

    type MyCompress = CompressionFunctionFromHasher<Val, MyHash, 2, 8>;
    let compress = MyCompress::new(hash);

    type ValMmcs = FieldMerkleTreeMmcs<Val, MyHash, MyCompress, 8>;
    let val_mmcs = ValMmcs::new(hash, compress);

    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    type Dft = Radix2Bowers;
    let dft = Dft::default();

    type Challenger = DuplexChallenger<Val, Perm16, 16>;

    type MyFriConfig = TwoAdicFriPcsConfig<Val, Challenge, Challenger, Dft, ValMmcs, ChallengeMmcs>;
    let fri_config = FriConfig {
        log_blowup: 1,
        num_queries: 40,
        proof_of_work_bits: 8,
        mmcs: challenge_mmcs,
    };

    type Pcs = TwoAdicFriPcs<MyFriConfig>;
    type MyConfig = StarkConfigImpl<Val, Challenge, PackedChallenge, Pcs, Challenger>;

    let pcs = Pcs::new(fri_config, dft, val_mmcs);

    let challenger = Challenger::new(perm16);
    let config = MyConfig::new(pcs, challenger);
    let proof = machine.prove(&config);

    let mut bytes = vec![];
    ciborium::into_writer(&proof, &mut bytes).expect("serialization failed");
    println!("Proof size: {} bytes", bytes.len());
    let deserialized_proof: MachineProof<MyConfig> =
        ciborium::from_reader(bytes.as_slice()).expect("deserialization failed");

    machine
        .verify(&config, &proof)
        .expect("verification failed");
    machine
        .verify(&config, &deserialized_proof)
        .expect("verification failed");

    // assert_eq!(machine.cpu().clock, 192);
    // assert_eq!(machine.cpu().operations.len(), 192);
    // assert_eq!(machine.mem().operations.values().flatten().count(), 401);
    // assert_eq!(machine.add_u32().operations.len(), 105);

    // assert_eq!(
    //     *machine.mem().cells.get(&(0x1000 + 4)).unwrap(), // Return value
    //     Word([0, 1, 37, 17,])                             // 25th fibonacci number (75025)
    // );
}
