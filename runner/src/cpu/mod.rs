use crate::{MachineState, Operands};
use p3_field::field::Field;
use valida_cpu::columns::CpuCols;
use valida_memory::columns::MemoryCols;

pub fn load32<F: Copy>(
    state: &mut MachineState<F>,
    ops: Operands,
    cpu_row: &mut CpuCols<F>,
    mem_rows: &mut Vec<MemoryCols<F>>,
) {
    let addr = state.memory.read(state.fp + ops.c);
    let cell = state.memory.read(addr.into());
    state.memory.write(state.fp + ops.a, cell);
    // TODO: Mutate rows to reflect the write
}

pub fn store32<F: Copy>(
    state: &mut MachineState<F>,
    ops: Operands,
    cpu_row: &mut CpuCols<F>,
    mem_rows: &mut Vec<MemoryCols<F>>,
) {
    let cell = state.memory.read(state.fp + ops.c);
    state.memory.write(state.fp + ops.b, cell);
    // TODO: Mutate rows to reflect the write
}

pub fn jal() {}

pub fn jalv() {}

pub fn beq() {}

pub fn bne() {}

pub fn imm32() {}
