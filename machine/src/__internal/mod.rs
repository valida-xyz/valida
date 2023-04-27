use crate::config::StarkConfig;
use p3_air::window::BasicAirWindow;
use p3_air::Air;
use p3_field::field::Field;
use p3_mersenne_31::Mersenne31;
use valida_machine::constraint_consumer::FoldingConstraintConsumer;

pub type DefaultField = Mersenne31;

pub fn prove<SC, A>()
where
    SC: StarkConfig,
    // for<'a> A: Air<
    //     SC::F,
    //     BasicAirWindow<'a, SC::F>,
    //     FoldingConstraintConsumer<SC::F, SC::FE, <SC::F as Field>::Packing>,
    // >,
{
}
