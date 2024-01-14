use crate::config::StarkConfig;
use crate::proof::ChipProof;
use crate::{Chip, Machine};

pub fn prove<M, A, SC>(
    _machine: &M,
    _config: &SC,
    _air: &A,
    _challenger: &mut SC::Challenger,
) -> ChipProof
where
    M: Machine<SC::Val>,
    A: Chip<M, SC>,
    SC: StarkConfig,
{
    // TODO: Sumcheck
    ChipProof
}
