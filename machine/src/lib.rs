extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::chip::Chip;
use p3_air::constraint_consumer::ConstraintConsumer;
use p3_air::types::AirTypes;
use p3_air::window::AirWindow;

pub mod bus;
pub mod chip;
pub mod config;
pub mod constraint_consumer;
pub mod proof;

pub trait Machine<T, W, CC>
where
    T: AirTypes,
    W: AirWindow<T::Var>,
    CC: ConstraintConsumer<T>,
{
    fn core_starks(&self) -> Vec<&dyn Chip<T, W, CC>> {
        vec![] // TODO
    }

    fn extension_starks(&self) -> Vec<&dyn Chip<T, W, CC>>;

    fn all_starks(&self) -> Vec<&dyn Chip<T, W, CC>> {
        let mut all = self.core_starks();
        all.extend(self.extension_starks());
        all
    }
}
