use crate::config::StarkConfig;
use core::marker::PhantomData;
use core::ops::{Add, Mul, Sub};
use p3_air::TwoRowMatrixView;
use p3_air::{Air, AirBuilder, PermutationAirBuilder};
use p3_field::Field;
use p3_field::{AbstractionOf, SymbolicField};
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31;

mod sym_var;
use sym_var::*;

pub type DefaultField = Mersenne31;

pub struct BasicFoldingAirBuilder<'a, F, Exp, Var> {
    main: TwoRowMatrixView<'a, Var>,
    is_first_row: Exp,
    is_last_row: Exp,
    is_transition: Exp,
    _phantom_f: PhantomData<F>,
}

pub fn prove<SC, A>(air: &A, trace: RowMajorMatrix<SC::F>)
where
    SC: StarkConfig,
    A: for<'a> Air<
        BasicFoldingAirBuilder<'a, SC::F, <SC::F as Field>::Packing, <SC::F as Field>::Packing>,
    >,
    A: for<'a> Air<
        BasicFoldingAirBuilder<
            'a,
            SC::F,
            SymbolicField<SC::F, BasicSymVar<SC::F>>,
            BasicSymVar<SC::F>,
        >,
    >,
{
}

impl<'a, F, Exp, Var> PermutationAirBuilder for BasicFoldingAirBuilder<'a, F, Exp, Var>
where
    F: Field,
    Exp:
        AbstractionOf<F> + Add<Var, Output = Exp> + Sub<Var, Output = Exp> + Mul<Var, Output = Exp>,
    Var: Into<Exp>
        + Copy
        + Add<F, Output = Exp>
        + Add<Var, Output = Exp>
        + Add<Exp, Output = Exp>
        + Sub<F, Output = Exp>
        + Sub<Var, Output = Exp>
        + Sub<Exp, Output = Exp>
        + Mul<F, Output = Exp>
        + Mul<Var, Output = Exp>
        + Mul<Exp, Output = Exp>,
{
    fn permutation(&self) -> TwoRowMatrixView<'a, Var> {
        self.main
    }

    fn permutation_randomness(&self) -> &[Exp] {
        todo!()
    }
}

impl<'a, F, Exp, Var> AirBuilder for BasicFoldingAirBuilder<'a, F, Exp, Var>
where
    F: Field,
    Exp:
        AbstractionOf<F> + Add<Var, Output = Exp> + Sub<Var, Output = Exp> + Mul<Var, Output = Exp>,
    Var: Into<Exp>
        + Copy
        + Add<F, Output = Exp>
        + Add<Var, Output = Exp>
        + Add<Exp, Output = Exp>
        + Sub<F, Output = Exp>
        + Sub<Var, Output = Exp>
        + Sub<Exp, Output = Exp>
        + Mul<F, Output = Exp>
        + Mul<Var, Output = Exp>
        + Mul<Exp, Output = Exp>,
{
    type F = F;
    type Exp = Exp;
    type Var = Var;
    type M = TwoRowMatrixView<'a, Var>;

    fn main(&self) -> Self::M {
        self.main
    }

    fn is_first_row(&self) -> Self::Exp {
        self.is_first_row.clone()
    }

    fn is_last_row(&self) -> Self::Exp {
        self.is_last_row.clone()
    }

    fn is_transition_window(&self, size: usize) -> Self::Exp {
        if size == 2 {
            self.is_transition.clone()
        } else {
            todo!()
        }
    }

    fn assert_zero<I: Into<Self::Exp>>(&mut self, x: I) {
        todo!()
    }
}
