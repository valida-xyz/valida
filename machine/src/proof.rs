use crate::config::StarkConfig;
use p3_commit::PCS;

pub struct MachineProof<C: StarkConfig> {
    pub opening_proof: <C::PCS as PCS<C::F>>::Proof,
}
