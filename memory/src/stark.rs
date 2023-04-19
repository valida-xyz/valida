use crate::cpu::columns::CpuCols;
use core::borrow::Borrow;
use p3_air::constraint_consumer::ConstraintConsumer;
use p3_air::types::AirTypes;
use p3_air::window::AirWindow;
use p3_air::Air;
use p3_field::field::Field;
use p3_matrix::Matrix;

pub struct MemoryStark;

impl<T, W, CC> Air<T, W, CC> for MemoryStark
where
    T: AirTypes,
    W: AirWindow<T::Var>,
    CC: ConstraintConsumer<T>,
{
    fn eval(&self, window: W, constraints: &mut CC) {
        todo!()
    }
}
