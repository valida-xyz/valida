//! Poseidon2 STARK Air Encoding

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
