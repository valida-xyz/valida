use crate::{Machine, Operands};
use p3_mersenne_31::Mersenne31 as Fp;

pub trait Instruction<M: Machine> {
    const OPCODE: u32;

    fn execute(state: &mut M, ops: Operands<Fp>);
}
