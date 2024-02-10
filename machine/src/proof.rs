use crate::config::StarkConfig;
use alloc::vec::Vec;
use core::fmt::Debug;
use p3_commit::Pcs;
use p3_matrix::dense::RowMajorMatrix;
use proptest::prelude::Arbitrary;
use proptest_derive::Arbitrary;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

type Val<SC> = <SC as StarkConfig>::Val;
type ValMat<SC> = RowMajorMatrix<Val<SC>>;
pub type Com<SC> = <<SC as StarkConfig>::Pcs as Pcs<Val<SC>, ValMat<SC>>>::Commitment;
pub type PcsProof<SC> = <<SC as StarkConfig>::Pcs as Pcs<Val<SC>, ValMat<SC>>>::Proof;

#[derive(Serialize, Deserialize, Arbitrary, Debug)]
#[serde(bound = "SC::Challenge: Serialize + DeserializeOwned")]
pub struct MachineProof<SC: StarkConfig>
        where Com<SC>: Arbitrary + Debug,
          PcsProof<SC>: Arbitrary + Debug {
    pub commitments: Commitments<Com<SC>>,
    pub opening_proof: PcsProof<SC>,
    pub chip_proofs: Vec<ChipProof<SC::Challenge>>,
}

#[derive(Serialize, Deserialize, Arbitrary, Debug)]
pub struct Commitments<Com: Arbitrary + Debug> {
    pub main_trace: Com,
    pub perm_trace: Com,
    pub quotient_chunks: Com,
}

#[derive(Serialize, Deserialize, Arbitrary, Debug)]
pub struct ChipProof<Challenge: Arbitrary + Debug> {
    pub log_degree: usize,
    pub opened_values: OpenedValues<Challenge>,
    pub cumulative_sum: Challenge,
}

#[derive(Serialize, Deserialize, Arbitrary, Debug)]
pub struct OpenedValues<Challenge: Arbitrary + Debug> {
    pub preprocessed_local: Vec<Challenge>,
    pub preprocessed_next: Vec<Challenge>,
    pub trace_local: Vec<Challenge>,
    pub trace_next: Vec<Challenge>,
    pub permutation_local: Vec<Challenge>,
    pub permutation_next: Vec<Challenge>,
    pub quotient_chunks: Vec<Challenge>,
}
