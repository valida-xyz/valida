# Valida

Valida is a STARK-based virtual machine aiming to improve upon the state of the art in the following categories:
- **Code reuse**: The VM has a RISC-inspired instruction set, simplifying the targeting of conventional programming languages. We are currently developing a backend compiler to compile LLVM IR to the Valida ISA. This will enable proving programs written in Rust, Go, C++, and others, and require minimal to no changes in source code.
- **Prover performance**: The VM is engineered to maximize prover performance. It is compatible with a 31-bit field, is restricted to degree 3 constraints, and has minimal instruction decoding. The VM doesn't contain any general-purpose registers, nor a dedicated stack, and instead operates directly on memory. We also utilize newer lookup arguments to reduce trace overhead involved in cross-chip communcation.
- **Extensibility**: The VM is designed to be customizable. It can easily be extended to include an arbitrary number of user-defined instructions. Procedural macros are used to construct the desired machine at compile time, avoiding any runtime penalties.

Our roadmap also includes implementing fast recursion and continuations to enable massively parallelizable proving of individual program execution.

## Interpreter
A standalone binary interpreter is available to execute Valida programs (skipping proof generation), and can be built by running `cargo build --release`.

For example, the 25th Fibonacci number can be returned by executing the command `printf "\x19" | ./target/release/valida basic/tests/data/fibonacci.bin | hexdump`.

## Tests
A proof of a Fibonacci program execution can be generated and tested for soundness by running `cargo test prove_fibonacci`.

## Compiler 
The Valida compiler is hosted separately, and can be found [here](https://github.com/valida-xyz/valida-compiler). For instructions on how to build Valida programs, please refer to the README in that repository.

## Backend
Valida uses the [Plonky3](https://github.com/Plonky3/Plonky3) toolkit to implement the STARK IOP, and to handle all field and cryptographic operations.

## Contributing
Valida is a community-driven project, and we encourage contributions to both the VM and compiler.

## License
The VM, compiler, and constraint generation code are open source, with no code obfuscation. The VM is licensed under the MIT and Apache licenses, while the compiler under the Apache license with LLVM Exceptions.
