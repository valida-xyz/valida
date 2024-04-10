use clap::Parser;
use std::fs;
use std::fs::File;
use std::io::{stdout, Write};

use valida_basic::BasicMachine;

use p3_baby_bear::BabyBear;

use p3_fri::{FriConfig, TwoAdicFriPcs, TwoAdicFriPcsConfig};
use valida_cpu::MachineWithCpuChip;
use valida_machine::{Machine, MachineProof, ProgramROM, StdinAdviceProvider};
use valida_memory::MachineWithMemoryChip;

use valida_elf::{load_executable_file, Program};
use valida_program::MachineWithProgramChip;
use valida_static_data::MachineWithStaticDataChip;

use p3_challenger::DuplexChallenger;
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_field::Field;
use p3_keccak::Keccak256Hash;
use p3_mds::coset_mds::CosetMds;
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_poseidon::Poseidon;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher32};
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use valida_machine::StarkConfigImpl;
use valida_machine::__internal::p3_commit::ExtensionMmcs;
use valida_output::MachineWithOutputChip;

use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

#[derive(Parser)]
struct Args {
    /// Command option either "run" or "prove" or "verify" or "interactive"
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

struct Context<'a> {
    machine_: BasicMachine<BabyBear>,
    args_: &'a Args,
    breakpoints_: Vec<u32>,
    stopped_: bool,
    last_fp_: u32,
    recorded_current_fp_: u32,
    last_fp_size_: u32,
}

impl Context<'_> {
    fn new(args: &Args) -> Context {
        let mut context = Context {
            machine_: BasicMachine::<BabyBear>::default(),
            args_: args.clone(),
            breakpoints_: Vec::new(),
            stopped_: false,
            last_fp_: args.stack_height,
            recorded_current_fp_: args.stack_height,
            last_fp_size_: 0,
        };

        let Program { code, data } = load_executable_file(
            fs::read(&args.program)
                .expect(format!("Failed to read executable file: {}", &args.program).as_str()),
        );

        context.machine_.program_mut().set_program_rom(&code);
        context.machine_.static_data_mut().load(data);
        context.machine_.cpu_mut().fp = args.stack_height;
        context.machine_.cpu_mut().save_register_state();

        context
    }

    fn step(&mut self) -> (bool, u32) {
        // do not execute if already stopped
        if self.stopped_ {
            return (true, 0);
        }
        let state = self.machine_.step(&mut StdinAdviceProvider);
        let pc = self.machine_.cpu().pc;
        let fp = self.machine_.cpu().fp;

        let instruction = self.machine_.program().program_rom.get_instruction(pc);
        println!("{:4} : {:?}", pc, instruction.to_string());

        // check if fp is changed
        if fp != self.recorded_current_fp_ {
            self.last_fp_size_ = self.recorded_current_fp_ - fp;
            self.last_fp_ = self.recorded_current_fp_;
        } else if fp == self.last_fp_ {
            self.last_fp_size_ = 0;
        }
        self.recorded_current_fp_ = fp;

        (state, pc)
    }
}

fn init_context(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    Ok(Some(String::from("created machine")))
}

fn status(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    // construct machine status
    let mut status = String::new();
    status.push_str("FP: ");
    status.push_str(&context.machine_.cpu().fp.to_string());
    status.push_str(", PC: ");
    status.push_str(&context.machine_.cpu().pc.to_string());
    status.push_str(if context.stopped_ {
        ", Stopped"
    } else {
        ", Running"
    });
    Ok(Some(status))
}

fn show_frame(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let size: i32 = match args.contains_id("size") {
        true => args
            .get_one::<String>("size")
            .unwrap()
            .parse::<i32>()
            .unwrap(),
        false => 6,
    };
    let mut frame = String::new();
    let fp = context.machine_.cpu().fp as i32;
    frame.push_str(format!("FP: {:x}\n", fp).as_str());
    for i in 0..size {
        let offset = i * -4;
        let read_addr = (fp + offset) as u32;
        let string_val = context.machine_.mem().examine(read_addr);
        let frameslot_addr = format!("{}(fp)", offset);
        let frameslot = format!("{:>7}", frameslot_addr);
        let frame_str = format!("\n{} : {}", frameslot, string_val);
        frame += &frame_str;
    }

    Ok(Some(frame))
}

fn last_frame(_: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let mut frame = String::new();

    let lfp = context.last_fp_;
    let fp = context.machine_.cpu().fp as i32;
    let last_size = context.last_fp_size_ as i32;
    frame += &format!("Last FP   : 0x{:x}, Frame size: {}\n", lfp, last_size).as_str();
    frame += &format!("Current FP: 0x{:x}\n", fp).as_str();

    // print last frame
    for i in (-5..(last_size / 4) + 1).rev() {
        let offset = (i * 4) as i32;
        let read_addr = (fp + offset) as u32;
        let string_val = context.machine_.mem().examine(read_addr);
        let frameslot_addr = format!("{}(fp)", offset);
        let frameslot = format!("0x{:<7x} | {:>7}", read_addr, frameslot_addr);
        let frame_str = format!("\n{} : {}", frameslot, string_val);
        frame += &frame_str;
    }
    Ok(Some(frame))
}

fn list_instrs(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let pc = context.machine_.cpu().pc;

    let program_rom = &context.machine_.program().program_rom;
    let total_size = program_rom.0.len();

    let print_size_arg = args.get_one::<String>("size");

    let print_size = match print_size_arg {
        Some(size) => size.parse::<u32>().unwrap(),
        None => 10,
    };

    let mut formatted = String::new();
    for i in 0..print_size {
        let cur_pc = pc + i;
        if cur_pc >= total_size as u32 {
            break;
        }
        let instruction = program_rom.get_instruction(cur_pc);
        formatted.push_str(format!("{:4} : {:?}\n", cur_pc, instruction.to_string()).as_str());
    }
    Ok(Some(formatted))
}

fn set_bp(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let pc = args
        .get_one::<String>("pc")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    context.breakpoints_.push(pc);
    let message = format!("Breakpoint set at pc: {}", pc);
    Ok(Some(message))
}
fn show_memory(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let addr = args
        .get_one::<String>("addr")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    // show memory at address, by default show 20 cells
    let mut memory = String::new();
    for i in 0..8 {
        let read_addr = addr + i * 4;
        let string_val = context.machine_.mem().examine(read_addr);
        let memory_str = format!("0x{:<8x} : {}\n", read_addr, string_val);
        memory += &memory_str;
    }

    Ok(Some(memory))
}

fn run_until(_: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let mut message = String::new();
    loop {
        let (stop, pc) = context.step();
        if stop {
            message.push_str("Execution stopped");
            break;
        }
        if context.breakpoints_.contains(&pc) {
            let bp_index = context.breakpoints_.iter().position(|&x| x == pc).unwrap();
            message = format!("Execution stopped at breakpoint {}, PC: {}", bp_index, pc);
            break;
        }
    }
    Ok(Some(message))
}

fn step(_: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let (stop, _) = context.step();
    if stop {
        context.stopped_ = true;
        Ok(Some(String::from("Execution stopped")))
    } else {
        Ok(None)
    }
}

fn repl_run(args: &Args) {
    // instantiate repl
    let mut repl = Repl::new(Context::new(args))
        .with_name("REPL")
        .with_version("v0.1.0")
        .with_description("Valida VM REPL")
        .with_banner("Start by using keywords")
        .with_command(Command::new("x").about("read machine state"), status)
        .with_command(
            Command::new("s")
                .arg(Arg::new("num_steps").required(false))
                .about("step assembly"),
            step,
        )
        .with_command(
            Command::new("f")
                .arg(Arg::new("size").required(false))
                .about("show frame"),
            show_frame,
        )
        .with_command(
            Command::new("lf").about("show last frame and current frame"),
            last_frame,
        )
        .with_command(
            Command::new("b")
                .arg(Arg::new("pc").required(false))
                .about("set break point at"),
            set_bp,
        )
        .with_command(
            Command::new("r").about("run until stop or breakpoint"),
            run_until,
        )
        .with_command(
            Command::new("l")
                .about("list instruction at current PC")
                .arg(Arg::new("size").required(false)),
            list_instrs,
        )
        .with_command(
            Command::new("m")
                .arg(Arg::new("addr").required(true))
                .about("show memory at address"),
            show_memory,
        )
        .with_command(
            Command::new("reset").about("reset machine state!"),
            init_context,
        );

    let _ = repl.run();
}

fn main() {
    let args = Args::parse();

    if args.action == "interactive" {
        repl_run(&args);
        return;
    }

    let mut machine = BasicMachine::<BabyBear>::default();
    let Program { code, data } = load_executable_file(
        fs::read(&args.program)
            .expect(format!("Failed to read executable file: {}", &args.program).as_str()),
    );
    machine.program_mut().set_program_rom(&code);
    machine.cpu_mut().fp = args.stack_height;
    machine.cpu_mut().save_register_state();
    machine.static_data_mut().load(data);

    // Run the program
    machine.run(&code, &mut StdinAdviceProvider);

    type Val = BabyBear;
    type Challenge = BinomialExtensionField<Val, 5>;
    type PackedChallenge = BinomialExtensionField<<Val as Field>::Packing, 5>;

    type Mds16 = CosetMds<Val, 16>;
    let mds16 = Mds16::default();

    type Perm16 = Poseidon<Val, Mds16, 16, 5>;
    let mut rng: Pcg64 = Seeder::from("validia seed").make_rng();
    let perm16 = Perm16::new_from_rng(4, 22, mds16, &mut rng);

    type MyHash = SerializingHasher32<Keccak256Hash>;
    let hash = MyHash::new(Keccak256Hash {});

    type MyCompress = CompressionFunctionFromHasher<Val, MyHash, 2, 8>;
    let compress = MyCompress::new(hash);

    type ValMmcs = FieldMerkleTreeMmcs<Val, MyHash, MyCompress, 8>;
    let val_mmcs = ValMmcs::new(hash, compress);

    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    type Dft = Radix2DitParallel;
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

    if args.action == "run" {
        let mut action_file;
        match File::create(args.action_file) {
            Ok(file) => {
                action_file = file;
            }
            Err(e) => {
                stdout().write(e.to_string().as_bytes()).unwrap();
                return ();
            }
        }
        action_file.write_all(&machine.output().bytes()).unwrap();
    } else if args.action == "prove" {
        let mut action_file;
        match File::create(args.action_file) {
            Ok(file) => {
                action_file = file;
            }
            Err(e) => {
                stdout().write(e.to_string().as_bytes()).unwrap();
                return ();
            }
        }
        let proof = machine.prove(&config);
        debug_assert!(machine.verify(&config, &proof).is_ok());
        let mut bytes = vec![];
        ciborium::into_writer(&proof, &mut bytes).expect("Proof serialization failed");
        action_file.write(&bytes).expect("Writing proof failed");
        stdout().write("Proof successful\n".as_bytes()).unwrap();
    } else if args.action == "verify" {
        let bytes = std::fs::read(args.action_file).expect("File reading failed");
        let proof: MachineProof<MyConfig> =
            ciborium::from_reader(bytes.as_slice()).expect("Proof deserialization failed");
        let verification_result = machine.verify(&config, &proof);
        match verification_result {
            Ok(_) => {
                stdout().write("Proof verified\n".as_bytes()).unwrap();
            }
            Err(_) => {
                stdout()
                    .write("Proof verification failed\n".as_bytes())
                    .unwrap();
            }
        }
    } else {
        stdout()
            .write("Action name unrecognized".as_bytes())
            .unwrap();
    }
}
