use crate::{Machine, ValidaAirBuilder};
use p3_air::{AirBuilder, PairBuilder, PermutationAirBuilder, TwoRowMatrixView};
use p3_field::{ExtensionField, Field};

pub struct ConstraintFolder<'a, F: Field, EF: ExtensionField<F>, M: Machine> {
    pub(crate) machine: &'a M,
    pub(crate) main: TwoRowMatrixView<'a, F>,
    pub(crate) preprocessed: TwoRowMatrixView<'a, F>,
    pub(crate) perm: TwoRowMatrixView<'a, EF>,
    pub(crate) rand_elems: &'a [EF],
    pub(crate) is_first_row: F,
    pub(crate) is_last_row: F,
    pub(crate) is_transition: F,
}

impl<'a, F, EF, M> PermutationAirBuilder for ConstraintFolder<'a, F, EF, M>
where
    F: Field,
    EF: ExtensionField<F>,
    M: Machine<EF = EF>,
{
    type EF = M::EF;
    type VarEF = M::EF;
    type ExprEF = M::EF;
    type MP = TwoRowMatrixView<'a, EF>;

    fn permutation(&self) -> Self::MP {
        self.perm
    }

    fn permutation_randomness(&self) -> &[Self::EF] {
        // TODO: implement
        self.rand_elems
    }
}

impl<'a, F, EF, M> PairBuilder for ConstraintFolder<'a, F, EF, M>
where
    F: Field,
    EF: ExtensionField<F>,
    M: Machine<EF = EF>,
{
    fn preprocessed(&self) -> Self::M {
        self.preprocessed
    }
}

impl<'a, M: Machine> ValidaAirBuilder for ConstraintFolder<'a, M::F, M::EF, M> {
    type Machine = M;

    fn machine(&self) -> &Self::Machine {
        self.machine
    }
}

impl<'a, F, EF, M> AirBuilder for ConstraintFolder<'a, F, EF, M>
where
    F: Field,
    EF: ExtensionField<F>,
    M: Machine,
{
    type F = F;
    type Expr = F;
    type Var = F;
    type M = TwoRowMatrixView<'a, F>;

    fn is_first_row(&self) -> Self::Expr {
        self.is_first_row
    }

    fn is_last_row(&self) -> Self::Expr {
        self.is_last_row
    }

    fn is_transition_window(&self, size: usize) -> Self::Expr {
        if size == 2 {
            self.is_transition
        } else {
            panic!("only supports a window size of 2")
        }
    }

    fn main(&self) -> Self::M {
        self.main
    }

    fn assert_zero<I: Into<Self::Expr>>(&mut self, _x: I) {
        // TODO
    }
}
