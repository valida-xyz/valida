use crate::config::StarkConfig;
use alloc::vec::Vec;
use p3_commit::PCS;
use p3_matrix::dense::RowMajorMatrix;

pub struct MachineProof<C: StarkConfig> {
    //pub opening_proof: <C::PCS as PCS<C::Val, RowMajorMatrix<C::Val>>>::Proof,
    pub chip_proofs: Vec<ChipProof>,
    pub phantom: core::marker::PhantomData<C>,
}

pub struct ChipProof;
