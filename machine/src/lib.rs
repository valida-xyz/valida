pub mod bus;
pub mod chip;
pub mod config;
pub mod constraint_consumer;
pub mod machine;
pub mod proof;
pub mod prover;
pub mod verifier;

use crate::framework::chip::Chip;
use crate::framework::config::Config;
use alloc::vec::Vec;

pub trait Machine<C: Config> {
    // fn core_starks(&self) -> Vec<&dyn Chip<C>> {
    //     todo!()
    // }
    //
    // fn extension_starks(&self) -> Vec<&dyn Chip<C>>;
    //
    // fn all_starks(&self) -> Vec<&dyn Chip<C>> {
    //     let mut all = self.core_starks();
    //     all.extend(self.extension_starks());
    //     all
    // }
}
