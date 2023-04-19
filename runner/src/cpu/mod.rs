use crate::{MachineState, Operands};
use p3_field::field::Field;
use valida_cpu::columns::CpuCols;
use valida_machine::{Addressable, Word};
use valida_memory::columns::{MemoryCols, ReadWriteLog};

/// LW / LOAD32
///
/// Follow the pointer stored at offset $c$ from the current frame pointer
/// and write the next 4 byte values to those beginning at offset a.
/// Operand $b$ is unused, but is constrained to $[c]$ in the trace.
pub fn load32<F: Addressable<F>>(
    state: &mut MachineState<F>,
    ops: Operands,
    cpu_row: &mut CpuCols<F>,
    mem_rows: &mut Vec<MemoryCols<F>>,
) {
    let read_addr_1 = state.fp + ops.c;
    let read_addr_2 = state.memory.read(read_addr_1);
    let write_addr = state.fp + ops.a;
    let cell = state.memory.read(read_addr_2.into());
    state.memory.write(write_addr, cell);
    state.pc += 1;

    mem_rows.push(MemoryCols::log_read(read_addr_1, read_addr_2));
    mem_rows.push(MemoryCols::log_read(read_addr_2, cell));
    mem_rows.push(MemoryCols::log_write(write_addr, cell));

    cpu_row.set_pc(state.pc.into());
    cpu_row.set_addr_read_1(read_addr_1.into());
    cpu_row.set_addr_read_2(read_addr_2.into());
    cpu_row.set_addr_write(write_addr.into());

    cpu_row.set_mem_read_1(read_addr_2);
    cpu_row.set_mem_read_2(cell);
    cpu_row.set_mem_write(cell);

    // TODO: Implement methods below
    cpu_row.set_opcode_flags(ops.as_slice());
    cpu_row.set_mem_channel_data();
}

/// SW / STORE32
///
/// Write the 4 byte values beginning at the address stored at offset $c$
/// to those beginning at offset $b$.
/// Operand $a$ is unused, but is constrained to $[c]$ in the trace.
pub fn store32<F: Addressable<F>>(
    state: &mut MachineState<F>,
    ops: Operands,
    cpu_row: &mut CpuCols<F>,
    mem_rows: &mut Vec<MemoryCols<F>>,
) {
    let read_addr = state.fp + ops.c;
    let write_addr = state.fp + ops.b;
    let cell = state.memory.read(read_addr);
    state.memory.write(write_addr, cell);
    state.pc += 1;

    mem_rows.push(MemoryCols::log_read(read_addr, cell));
    mem_rows.push(MemoryCols::log_write(write_addr, cell));

    cpu_row.set_pc(state.pc.into());
    cpu_row.set_addr_read_1(read_addr.into());
    cpu_row.set_addr_write(write_addr.into());

    cpu_row.set_mem_read_1(cell);
    cpu_row.set_mem_write(cell);

    cpu_row.set_opcode_flags(ops.as_slice());
    cpu_row.set_mem_channel_data();
}

pub fn jal() {}

pub fn jalv() {}

pub fn beq() {}

pub fn bne() {}

pub fn imm32() {}
