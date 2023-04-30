use crate::config::StarkConfig;
use p3_air::window::BasicAirWindow;
use p3_air::Air;
use p3_mersenne_31::Mersenne31;

pub type DefaultField = Mersenne31;

pub fn prove<SC, A>()
where
    SC: StarkConfig,
    for<'a> A: Air<SC::F, BasicAirWindow<'a, SC::F>>,
{
}
