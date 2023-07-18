use crate::__internal::ConstraintFolder;
use crate::config::StarkConfig;
use crate::proof::ChipProof;
use crate::{Chip, Machine};
use p3_air::Air;

pub fn prove<M, A, SC>(_machine: &M, _config: &SC, _air: &A, _challenger: &mut SC::Chal) -> ChipProof
where
    M: Machine,
    A: for<'a> Air<ConstraintFolder<'a, M::F, M::EF, M>> + Chip<M>,
    SC: StarkConfig<Val = M::F, Challenge = M::EF>,
{
    // TODO: Sumcheck
    ChipProof
}
