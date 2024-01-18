use crate::{Chip, Machine};
use alloc::vec::Vec;
use p3_uni_stark::{Proof, StarkConfig};
pub struct MachineProof<C: StarkConfig> {
    //pub opening_proof: <C::PCS as PCS<C::Val, RowMajorMatrix<C::Val>>>::Proof,
    pub chip_proof: ChipProof<C>,
    pub phantom: core::marker::PhantomData<C>,
}

pub struct ChipProof<C: StarkConfig> {
    pub proof: Proof<C>,
}
