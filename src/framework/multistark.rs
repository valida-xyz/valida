use crate::framework::config::Config;
use crate::framework::stark::Stark;
use alloc::vec::Vec;

pub trait VmConfig<C: Config> {
    fn core_starks(&self) -> Vec<&dyn Stark<C>> {
        todo!()
    }

    fn extension_starks(&self) -> Vec<&dyn Stark<C>>;

    fn all_starks(&self) -> Vec<&dyn Stark<C>> {
        let mut all = self.core_starks();
        all.extend(self.extension_starks());
        all
    }
}
