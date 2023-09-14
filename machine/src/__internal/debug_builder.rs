use crate::{Machine, ValidaAirBuilder};
use p3_air::{AirBuilder, PairBuilder, PermutationAirBuilder, TwoRowMatrixView};
use p3_field::{ExtensionField, Field};

/// An `AirBuilder` which asserts that each constraint is zero, allowing any failed constraints to
/// be detected early.
pub struct DebugConstraintBuilder<'a, F: Field, EF: ExtensionField<F>, M: Machine> {
    pub(crate) machine: &'a M,
    pub(crate) main: TwoRowMatrixView<'a, F>,
    pub(crate) preprocessed: TwoRowMatrixView<'a, F>,
    pub(crate) perm: TwoRowMatrixView<'a, EF>,
    pub(crate) perm_challenges: &'a [EF],
    pub(crate) is_first_row: F,
    pub(crate) is_last_row: F,
    pub(crate) is_transition: F,
}

impl<'a, F, EF, M> PermutationAirBuilder for DebugConstraintBuilder<'a, F, EF, M>
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
        self.perm_challenges
    }
}

impl<'a, F, EF, M> PairBuilder for DebugConstraintBuilder<'a, F, EF, M>
where
    F: Field,
    EF: ExtensionField<F>,
    M: Machine<EF = EF>,
{
    fn preprocessed(&self) -> Self::M {
        self.preprocessed
    }
}

impl<'a, M: Machine> ValidaAirBuilder for DebugConstraintBuilder<'a, M::F, M::EF, M> {
    type Machine = M;

    fn machine(&self) -> &Self::Machine {
        self.machine
    }
}

impl<'a, F, EF, M> AirBuilder for DebugConstraintBuilder<'a, F, EF, M>
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

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
        assert_eq!(x.into(), F::ZERO, "constraints must evaluate to zero");
    }
}
