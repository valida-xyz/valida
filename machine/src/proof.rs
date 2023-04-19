use crate::framework::config::Config;
use p3_commit::pcs::PCS;

pub struct MachineProof<C: Config> {
    pub opening_proof: <C::PCS as PCS<C::F>>::Proof,
}
