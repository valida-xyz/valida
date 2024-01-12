use crate::__internal::ConstraintFolder;
use crate::proof::ChipProof;
use crate::{Chip, Machine};
use p3_air::Air;
use p3_uni_stark::{prove as stark_prove,StarkConfig,SymbolicAirBuilder,ProverConstraintFolder};
/*
pub fn prove<M, A, SC>(
    machine: &M,
    config: &SC,
    air: &A,
    challenger: &mut SC::Challenger,
) -> ChipProof<SC>
where
    M: Machine,
    A: for<'a> Air<ProverConstraintFolder<'a, SC>> + Chip<M> + Air<SymbolicAirBuilder<SC::Val>>,
    SC: StarkConfig<Val = M::F, Challenge = M::EF>,
{
    let trace = air.generate_trace(&machine);
    let proof = stark_prove(config,air,challenger,trace);

    ChipProof{
	proof
    }
}
*/
