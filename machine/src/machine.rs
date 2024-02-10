use crate::config::StarkConfig;
use crate::program::ProgramROM;
use crate::proof::{ Com, MachineProof, PcsProof };
use crate::AdviceProvider;
use core::fmt::Debug;
use p3_field::Field;
use proptest::prelude::Arbitrary;

pub trait Machine<F: Field + Arbitrary + Debug>: Sync {
    fn run<Adv>(&mut self, program: &ProgramROM<i32>, advice: &mut Adv)
    where
        Adv: AdviceProvider;

    fn prove<SC>(&self, config: &SC) -> MachineProof<SC>
    where
        SC: StarkConfig<Val = F>,
        Com<SC>: Arbitrary + Debug,
        PcsProof<SC>: Arbitrary + Debug;

    fn verify<SC>(&self, config: &SC, proof: &MachineProof<SC>) -> Result<(), ()>
    where
        SC: StarkConfig<Val = F>,
        Com<SC>: Arbitrary + Debug,
        PcsProof<SC>: Arbitrary + Debug;
}
