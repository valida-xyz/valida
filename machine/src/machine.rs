use crate::config::StarkConfig;
use crate::program::ProgramROM;
use crate::proof::MachineProof;
use crate::AdviceProvider;
use p3_commit::Pcs;
use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;

#[derive(PartialEq, Eq)]
pub enum StoppingFlag {
    DidStop,
    DidNotStop,
}

#[derive(Debug)]
pub enum FailureReason<SC: StarkConfig> {
    CumulativeSumNonZero,
    FailureToVerifyMultiOpening(<<SC as StarkConfig>::Pcs as Pcs<SC::Val, RowMajorMatrix<SC::Val>>>::Error),
}

pub trait Machine<F: Field>: Sync {
    fn run<Adv>(&mut self, program: &ProgramROM<i32>, advice: &mut Adv)
    where
        Adv: AdviceProvider;

    fn step<Adv>(&mut self, advice: &mut Adv) -> StoppingFlag
    where
        Adv: AdviceProvider;

    fn prove<SC>(&self, config: &SC) -> MachineProof<SC>
    where
        SC: StarkConfig<Val = F>;

    fn verify<SC>(&self, config: &SC, proof: &MachineProof<SC>) -> Result<(), FailureReason<SC>>
    where
        SC: StarkConfig<Val = F>;
}
