extern crate core;

use p3_baby_bear::BabyBear;
use p3_fri::{TwoAdicFriPcs, TwoAdicFriPcsConfig};
use valida_basic_macro::BasicMachine;
use valida_cpu::{
    BneInstruction, Imm32Instruction, Load32Instruction, MachineWithCpuChip, StopInstruction,
};
use valida_machine::{
    FixedAdviceProvider, Instruction, InstructionWord, Machine, Operands, ProgramROM, Word,
};

use valida_program::MachineWithProgramChip;
use valida_static_data::MachineWithStaticDataChip;

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

#[test]
fn prove_static_data() {
    // _start:
    //  imm32 0(fp), 0, 0, 0, 0x10
    //  load32 -4(fp), 0(fp), 0, 0, 0
    //  bnei _start, 0(fp), 0x25, 0, 1        // infinite loop unless static value is loaded
    //  stop
    let program = vec![
        InstructionWord {
            opcode: <Imm32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([0, 0, 0, 0, 0x10]),
        },
        InstructionWord {
            opcode: <Load32Instruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([-4, 0, 0, 0, 0]),
        },
        InstructionWord {
            opcode: <BneInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands([0, -4, 0x25, 0, 1]),
        },
        InstructionWord {
            opcode: <StopInstruction as Instruction<BasicMachine<Val>, Val>>::OPCODE,
            operands: Operands::default(),
        },
    ];

    let mut machine = BasicMachine::<Val>::default();
    let rom = ProgramROM::new(program);
    machine.static_data_mut().write(0x10, Word([0, 0, 0, 0x25]));
    machine.static_data_mut().write(0x14, Word([0, 0, 0, 0x32]));
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
    machine
        .verify(&config, &proof)
        .expect("verification failed");
}
