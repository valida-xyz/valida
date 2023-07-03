use crate::__internal::ConstraintFolder;
use crate::config::StarkConfig;
use crate::{Chip, Machine};
use p3_air::Air;
use p3_challenger::Challenger;
use p3_commit::PCS;
use p3_matrix::dense::RowMajorMatrix;

pub fn prove<M, A, SC>(
    machine: &M,
    config: &SC,
    air: &A,
    _challenger: &mut SC::Chal,
    main: RowMajorMatrix<M::F>,
    perm: RowMajorMatrix<M::EF>,
) where
    M: Machine,
    SC: StarkConfig<Val = M::F, Challenge = M::EF>,
    A: for<'a> Air<ConstraintFolder<'a, M::F, M::EF, M>> + Chip<M>,
{
    let (_trace_commit, _trace_data) = config.pcs().commit_batch(main);
}
