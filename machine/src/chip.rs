use crate::{Machine, Operands};
use alloc::vec;
use alloc::vec::Vec;

use p3_air::VirtualPairCol;
use p3_field::ExtensionField;
use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31 as Fp;

const LOOKUP_DEGREE_BOUND: usize = 3;

pub trait Chip<M: Machine> {
    /// Generate the main trace for the chip given the provided machine.
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F>;

    fn local_sends(&self) -> Vec<Interaction<M::F>> {
        vec![]
    }

    fn local_receives(&self) -> Vec<Interaction<M::F>> {
        vec![]
    }

    fn global_sends(&self, machine: &M) -> Vec<Interaction<M::F>> {
        vec![]
    }

    fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
        vec![]
    }
}

pub struct Interaction<F: Field> {
    pub fields: Vec<VirtualPairCol<F>>,
    pub count: VirtualPairCol<F>,
    pub argument_index: usize,
}

/// Generate the permutation trace for the chip given the provided machine.
fn generate_permutation_trace<M: Machine, C: Chip<M>, EF: ExtensionField<M::F>>(
    chip: &C,
    machine: &M,
    main_trace: RowMajorMatrix<M::F>,
    random_elements: Vec<EF>,
) -> RowMajorMatrix<M::F> {
    // LogUp::<NUM_MEM_LOOKUPS, LOOKUP_DEGREE_BOUND>::new(MEM_LOOKUPS)
    //     .build_trace(&main_trace, random_elements)
    todo!()
}

#[macro_export]
macro_rules! instructions {
    ($($t:ident),*) => {
        $(
            #[derive(Default)]
            pub struct $t {}
        )*
    }
}

pub trait Instruction<M: Machine> {
    const OPCODE: u32;

    fn execute(state: &mut M, ops: Operands<i32>);
}
