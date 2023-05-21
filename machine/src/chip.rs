use crate::{Machine, Operands};

use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31 as Fp;

pub trait Chip<M: Machine> {
    /// Generate the main trace for the chip given the provided machine.
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F>;

    // fn bus_senders(&self) -> Vec<BusInteraction> {}
    // fn bus_receivers(&self) -> Vec<BusInteraction> {}
    // fn permutation_pairs(&self) -> Vec<PermutationPair> {}

    ///// Generate the permutation trace for the chip given the provided machine.
    //fn generate_permutation_trace(
    //    &self,
    //    machine: &M,
    //    main_trace: RowMajorMatrix<Self::F>,
    //    random_elements: Vec<Self::FE>,
    //) -> RowMajorMatrix<Self::F>;
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

    fn execute(state: &mut M, ops: Operands<Fp>);
}
