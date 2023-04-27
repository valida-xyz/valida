use crate::{Machine, Operands};

pub trait Instruction<M: Machine> {
    const OPCODE: u32;

    fn execute(state: &mut M, ops: Operands);
}
