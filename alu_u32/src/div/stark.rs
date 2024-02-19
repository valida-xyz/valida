use alloc::vec::Vec;
use super::Div32Chip;
use super::columns::Div32Cols;
use crate::mul::{stark::{mul_builder, sigma_m}};
use core::borrow::Borrow;
use crate::div::columns::NUM_DIV_COLS;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField};
use p3_matrix::MatrixRowSlices;
use valida_machine::Word;
impl<F> BaseAir<F> for Div32Chip {
    fn width(&self) -> usize {
        NUM_DIV_COLS
    }
}

impl<F, AB> Air<AB> for Div32Chip
where
    F: PrimeField,
    AB: AirBuilder<F = F>,
{
    fn eval(&self, builder: &mut AB) {
	//Keeping this function intentionally empty, as we will be invoking Mul32, Sub32, and Lt32 to prove that input_1 - input2*output < output, as this implies that input_1 < input_2*output + output -> input_1 = input_2*output + q for q < output. 
    }
}
