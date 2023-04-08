use crate::framework::config::Config;
use crate::framework::constraint_consumer::ConstraintConsumer;
use crate::framework::window::AirWindow;
use p3_field::field::Field;
use p3_field::trivial_extension::TrivialExtension;

pub trait Stark<C: Config> {
    // TODO: Is it actually needed?
    fn columns(&self) -> usize;

    fn eval_packed_base(
        &self,
        vars: AirWindow<<C::F as Field>::Packing>,
        constraints: &mut ConstraintConsumer<C::F, C::FE, <C::F as Field>::Packing>,
    );

    fn eval_ext(
        &self,
        vars: AirWindow<C::FE>,
        constraints: &mut ConstraintConsumer<C::FE, TrivialExtension<C::FE>, C::FE>,
    );
}
