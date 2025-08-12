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
    pub chip_proofs: Vec<ChipProof<SC>>,
}

#[derive(Serialize, Deserialize)]
pub struct Commitments<Com> {
    pub main_trace: Com,
    pub perm_trace: Com,
    pub quotient_chunks: Com,
}

#[derive(Serialize, Deserialize)]
pub struct ColumnIndex(usize);

#[derive(Serialize, Deserialize)]
pub struct ColumnVector<A>(Vec<A>);

#[derive(Serialize, Deserialize)]
pub struct ChipProof<SC: StarkConfig> {
    pub public_inputs: Vec<(ColumnIndex, ColumnVector<SC::Val>)>,
    pub log_degree: usize,
    pub opened_values: OpenedValues<SC::Challenge>,
    pub cumulative_sum: SC::Challenge,
}

#[derive(Serialize, Deserialize)]
pub struct OpenedValues<Challenge> {
    pub preprocessed_local: Vec<Challenge>,
    pub preprocessed_next: Vec<Challenge>,
    pub trace_local: Vec<Challenge>,
    pub trace_next: Vec<Challenge>,
    pub permutation_local: Vec<Challenge>,
    pub permutation_next: Vec<Challenge>,
    pub quotient_chunks: Vec<Challenge>,
}
