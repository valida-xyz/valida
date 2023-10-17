use std::io::Cursor;
use std::io::Read;
use std::process::{Command, Stdio};

use byteorder::{LittleEndian, WriteBytesExt};

#[test]
fn run_fibonacci() {
    // Execute the fibonacci binary
    let filepath = "tests/programs/binary/fibonacci.bin";
    let fib_number = 25;
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "valida", filepath])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process");
    let stdin = child.stdin.as_mut().expect("failed to get stdin");
    stdin.write_u32::<LittleEndian>(fib_number).unwrap();

    // Compare stdout with the expected value in the Fibonacci sequence
    let value = fibonacci(fib_number);
    let output = child.wait_with_output().expect("failed to wait on child");
    let mut cursor = Cursor::new(output.stdout);
    let mut buf = [0; 4];
    cursor.read_exact(&mut buf).unwrap();
    let result = u32::from_le_bytes(buf);
    assert_eq!(result, value);
}

fn fibonacci(n: u32) -> u32 {
    let mut a = 0u32;
    let mut b = 1u32;
    for _ in 0..n {
        let temp = a;
        a = b;
        (b, _) = temp.overflowing_add(b);
    }
    a
}
