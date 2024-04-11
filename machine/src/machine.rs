use crate::config::StarkConfig;
use crate::program::ProgramROM;
use crate::proof::MachineProof;
use crate::AdviceProvider;
use p3_field::Field;

#[derive(PartialEq, Eq)]
pub enum StoppingFlag {
    DidStop,
    DidNotStop,
}

#[derive(Debug)]
pub enum FailureReason {
    CumulativeSumNonZero,
    FailureToVerifyMultiOpening,
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

    fn verify<SC>(&self, config: &SC, proof: &MachineProof<SC>) -> Result<(), FailureReason>
    where
        SC: StarkConfig<Val = F>;
}
