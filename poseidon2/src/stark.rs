//! Poseidon2 STARK Air Encoding

use crate::columns::Columns;
use core::borrow::Borrow;
use p3_air::Air;
use p3_air::AirBuilder;
use p3_field::PrimeField;
use p3_matrix::Matrix;

///
pub struct Stark;

impl<AB> Air<AB> for Stark
where
    AB: AirBuilder,
    AB::F: PrimeField,
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();

        let local: &Columns<AB::Var> = main.row(0).borrow();
        let next: &Columns<AB::Var> = main.row(1).borrow();

        todo!()
    }
}
