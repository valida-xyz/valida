use clap::Parser;
use std::io::{stdout, Write, Read};
use std::fs::File;

use valida_basic::BasicMachine;

use p3_baby_bear::BabyBear;

use p3_fri::{TwoAdicFriPcs, TwoAdicFriPcsConfig, FriConfig};
use valida_cpu::MachineWithCpuChip;
use valida_machine::{ Machine, MachineProof, ProgramROM, StdinAdviceProvider,
};

use valida_program::MachineWithProgramChip;

use p3_challenger::DuplexChallenger;
use p3_dft::Radix2Bowers;
use p3_field::extension::BinomialExtensionField;
use p3_field::Field;
use p3_keccak::Keccak256Hash;
use p3_mds::coset_mds::CosetMds;
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_poseidon::Poseidon;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher32};
use rand::thread_rng;
use valida_machine::StarkConfigImpl;
use valida_machine::__internal::p3_commit::ExtensionMmcs;
use valida_output::MachineWithOutputChip;

#[derive(Parser)]
struct Args {
    /// Command option either "run" or "prove" or "verify"
    #[arg(name = "Action Option")]
    action: String,

    /// Program binary file
    #[arg(name = "PROGRAM FILE")]
    program: String,

    /// The output file for run or prove, or the input file for verify
    #[arg(name = "ACTION FILE")]
    action_file: String,

    /// Stack height (which is also the initial frame pointer value)
    #[arg(long, default_value = "16777216")]
    stack_height: u32,
}

type Val = BabyBear;
type Challenge = BinomialExtensionField<Val, 5>;
type PackedChallenge = BinomialExtensionField<<Val as Field>::Packing, 5>;
type Perm16 = Poseidon<Val, Mds16, 16, 5>;
type Mds16 = CosetMds<Val, 16>;
type MyHash = SerializingHasher32<Keccak256Hash>;
type MyCompress = CompressionFunctionFromHasher<Val, MyHash, 2, 8>;
type ValMmcs = FieldMerkleTreeMmcs<Val, MyHash, MyCompress, 8>;
type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
type Dft = Radix2Bowers;
type Challenger = DuplexChallenger<Val, Perm16, 16>;
type MyFriConfig = TwoAdicFriPcsConfig<Val, Challenge, Challenger, Dft, ValMmcs, ChallengeMmcs>;
type Pcs = TwoAdicFriPcs<MyFriConfig>;
type MyConfig = StarkConfigImpl<Val, Challenge, PackedChallenge, Pcs, Challenger>;

fn main() {
    let args = Args::parse();

    let mut machine = BasicMachine::<BabyBear>::default();
    let rom = match ProgramROM::from_file(&args.program) {
        Ok(contents) => contents,
        Err(e) => panic!("Failure to load file: {}. {}", &args.program, e),
    };
    machine.program_mut().set_program_rom(&rom);
    machine.cpu_mut().fp = args.stack_height;
    machine.cpu_mut().save_register_state();

    // Run the program
    machine.run(&rom, &mut StdinAdviceProvider);

    let mds16 = Mds16::default();

    let perm16 = Perm16::new_from_rng(4, 22, mds16, &mut thread_rng()); // TODO: Use deterministic RNG

    let hash = MyHash::new(Keccak256Hash {});

    let compress = MyCompress::new(hash);

    let val_mmcs = ValMmcs::new(hash, compress);

    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    let dft = Dft::default();

    let fri_config = FriConfig {
        log_blowup: 1,
        num_queries: 40,
        proof_of_work_bits: 8,
        mmcs: challenge_mmcs,
    };

    let pcs = Pcs::new(fri_config, dft, val_mmcs);

    let challenger = Challenger::new(perm16);
    let config = MyConfig::new(pcs, challenger);

    if args.action == "run" {
        let mut action_file;
        match File::create(args.action_file) {
            Ok(file) => {action_file = file;},
            Err(e) => {stdout().write(e.to_string().as_bytes()).unwrap(); return ()},
        }
        action_file.write_all(&machine.output().bytes()).unwrap();
    } else if args.action == "prove" {
        let mut action_file;
        match File::create(args.action_file) {
            Ok(file) => {action_file = file;},
            Err(e) => {stdout().write(e.to_string().as_bytes()).unwrap(); return ()},
        }
        let proof = machine.prove(&config);
        let mut bytes = vec![];
        ciborium::into_writer(&proof, &mut bytes).expect("Proof serialization failed");
        action_file.write(&bytes).expect("Writing proof failed");
        stdout().write("Proof successful\n".as_bytes()).unwrap();
    } else if args.action == "verify" {
        let bytes = std::fs::read(args.action_file).expect("File reading failed");
        let proof: MachineProof<MyConfig> = ciborium::from_reader(bytes.as_slice()).expect("Proof deserialization failed");
        let proof2 = machine.prove(&config); // TODO: delete this line
        machine.verify(&config, &proof2).expect("Proof 2 verification failed"); // TODO: delete this line
        let mut bytes2 = vec![]; // TODO: delete this line
        ciborium::into_writer(&proof, &mut bytes2).expect("Proof 2 serialization failed"); // TODO: delete this line
        std::println!("bytes2.len() == {}", bytes2.len()); // TODO: delete this line
        let proof3: MachineProof<MyConfig> = ciborium::from_reader(bytes2.as_slice()).expect("Proof 3 deserialization failed"); // TODO: delete this line
        machine.verify(&config, &proof3).expect("Proof 3 verification failed"); // TODO: delete this line
        let verification_result = machine.verify(&config, &proof);
        match verification_result {
            Ok(_) => {stdout().write("Proof verified\n".as_bytes()).unwrap();},
            Err(_) => {stdout().write("Proof verification failed\n".as_bytes()).unwrap();}
        }
    } else {
        stdout().write("Action name unrecognized".as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use crate::{ MachineProof, MyConfig };
    proptest! {
        #[test]
        fn serialization_roundtrip(proof: MachineProof<MyConfig>) {
            let mut bytes = vec![];
            ciborium::into_writer(&proof, &mut bytes);
            let proof2: MachineProof<MyConfig> = ciborium::from_reader(bytes.as_slice()).expect("Proof deserialization failed");
            assert!(proof2 == proof);
        }
    }
}
