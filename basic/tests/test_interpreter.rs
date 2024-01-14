use p3_baby_bear::BabyBear;
use std::fs::read_to_string;
use valida_assembler::assemble;
use valida_basic::BasicMachine;
use valida_cpu::MachineWithCpuChip;
use valida_machine::{FixedAdviceProvider, Machine, ProgramROM};
use valida_output::MachineWithOutputChip;
use valida_program::MachineWithProgramChip;

#[test]
fn run_fibonacci() {
    let mut machine = BasicMachine::<BabyBear>::default();
    let asm_path = "tests/programs/assembly/fibonacci.val";
    let asm = read_to_string(asm_path).expect("Failed to read asm");
    let rom = ProgramROM::from_machine_code(&assemble(&asm).unwrap());
    machine.program_mut().set_program_rom(&rom);
    machine.cpu_mut().fp = 16777216; // default stack height
    machine.cpu_mut().save_register_state();

    let fib_number = 25;
    // Put the desired fib number in the advice tape.
    let mut advice = FixedAdviceProvider::new(vec![fib_number]);

    // Run the program
    machine.run(&rom, &mut advice);
    let output = machine.output().bytes();
    assert_eq!(output.len(), 4);
    let actual_result = u32::from_le_bytes(output.try_into().unwrap());

    let expected_result = fibonacci(fib_number);
    assert_eq!(actual_result, expected_result);
}

fn fibonacci(n: u8) -> u32 {
    let mut a = 0u32;
    let mut b = 1u32;
    for _ in 0..n {
        let temp = a;
        a = b;
        (b, _) = temp.overflowing_add(b);
    }
    a
}
