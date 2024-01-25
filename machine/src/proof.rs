use crate::config::StarkConfig;
use alloc::vec::Vec;
use p3_commit::Pcs;
use p3_matrix::dense::RowMajorMatrix;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

type Val<SC> = <SC as StarkConfig>::Val;
type ValMat<SC> = RowMajorMatrix<Val<SC>>;
type Com<SC> = <<SC as StarkConfig>::Pcs as Pcs<Val<SC>, ValMat<SC>>>::Commitment;
type PcsProof<SC> = <<SC as StarkConfig>::Pcs as Pcs<Val<SC>, ValMat<SC>>>::Proof;

#[derive(Serialize, Deserialize)]
#[serde(bound = "SC::Challenge: Serialize + DeserializeOwned")]
pub struct MachineProof<SC: StarkConfig> {
    pub commitments: Commitments<Com<SC>>,
    pub opening_proof: PcsProof<SC>,
    pub chip_proofs: Vec<ChipProof<SC::Challenge>>,
}

#[derive(Serialize, Deserialize)]
pub struct Commitments<Com> {
    pub main_trace: Com,
    pub perm_trace: Com,
    pub quotient_chunks: Com,
}

#[derive(Serialize, Deserialize)]
pub struct ChipProof<Challenge> {
    pub(crate) log_degree: usize,
    pub(crate) opened_values: OpenedValues<Challenge>,
}

#[derive(Serialize, Deserialize)]
pub struct OpenedValues<Challenge> {
    pub(crate) preprocessed_local: Vec<Challenge>,
    pub(crate) preprocessed_next: Vec<Challenge>,
    pub(crate) trace_local: Vec<Challenge>,
    pub(crate) trace_next: Vec<Challenge>,
    pub(crate) permutation_local: Vec<Challenge>,
    pub(crate) permutation_next: Vec<Challenge>,
    pub(crate) quotient_chunks: Vec<Challenge>,
}
