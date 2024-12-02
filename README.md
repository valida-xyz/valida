# Valida zkVM (Archived Repository)

This repository has been **archived** and is no longer maintained. The development and maintenance of Valida zkVM have moved to a new repository hosted on Lita's GitHub.

For the latest updates and contributions, please visit the new repository here:

**[New Valida zkVM Repository](https://github.com/lita-xyz/valida-releases)**

Thank you for your continued interest and contributions to Valida zkVM!

---

Valida is a STARK-based virtual machine aiming to improve upon the state of the art in the following categories:
- **Code reuse**: The VM has a RISC-inspired instruction set, simplifying the targeting of conventional programming languages. We are currently developing a backend compiler to compile LLVM IR to the Valida ISA. This will enable proving programs written in Rust, Go, C++, and others, and require minimal to no changes in source code.
- **Prover performance**: The VM is engineered to maximize prover performance. It is compatible with a 31-bit field, is restricted to degree 3 constraints, and has minimal instruction decoding. The VM doesn't contain any general-purpose registers, nor a dedicated stack, and instead operates directly on memory. We also utilize newer lookup arguments to reduce trace overhead involved in cross-chip communication.
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

To use a local copy of Plonky3 instead of the pinned version of Plonky3, you can run:

```bash
mkdir -p .cargo
for lib in air baby-bear commit challenger dft field fri goldilocks keccak matrix maybe-rayon mds merkle-tree poseidon symmetric uni-stark util
do
echo "patch.\"https://github.com/valida-xyz/Plonky3.git\".p3-$lib.path = \"../Plonky3/$lib\"" >> .cargo/config.toml
done
```

After adding this configuration, just run `cargo build` and other `cargo` commands normally, and they will use your local copy of Plonky3. To revert this configuration change, you can just `rm .cargo/config.toml` (assuming you have no other configuration in that file that you want to keep).

## Contributing
Valida is a community-driven project, and we encourage contributions to both the VM and compiler.

## License
The VM, compiler, and constraint generation code are open source, with no code obfuscation. The VM is licensed under the MIT and Apache licenses, while the compiler under the Apache license with LLVM Exceptions.
