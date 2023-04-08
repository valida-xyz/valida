use crate::framework::config::Config;
use crate::framework::stark::Stark;

pub trait MultiStark<C: Config, const N: usize> {
    fn starks(&self) -> [&dyn Stark<C>; N];
}
