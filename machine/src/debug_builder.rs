use crate::{Machine, ValidaAirBuilder};
use p3_air::{AirBuilder, PairBuilder, PermutationAirBuilder, TwoRowMatrixView};
use p3_field::AbstractField;
use valida_machine::StarkConfig;
/// An `AirBuilder` which asserts that each constraint is zero, allowing any failed constraints to
/// be detected early.
pub struct DebugConstraintBuilder<'a, M: Machine<SC::Val>, SC: StarkConfig> {
    pub(crate) machine: &'a M,
    pub(crate) main: TwoRowMatrixView<'a, SC::Val>,
    pub(crate) preprocessed: TwoRowMatrixView<'a, SC::Val>,
    pub(crate) perm: TwoRowMatrixView<'a, SC::Challenge>,
    pub(crate) perm_challenges: &'a [SC::Challenge],
    pub(crate) is_first_row: SC::Val,
    pub(crate) is_last_row: SC::Val,
    pub(crate) is_transition: SC::Val,
}

impl<'a, M, SC> AirBuilder for DebugConstraintBuilder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type F = SC::Val;
    type Expr = SC::Val;
    type Var = SC::Val;
    type M = TwoRowMatrixView<'a, SC::Val>;

    fn main(&self) -> Self::M {
        self.main
    }

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

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
        assert_eq!(
            x.into(),
            SC::Val::zero(),
            "constraints must evaluate to zero"
        );
    }
}

impl<'a, M, SC> PairBuilder for DebugConstraintBuilder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    fn preprocessed(&self) -> Self::M {
        self.preprocessed
    }
}

impl<'a, M, SC> PermutationAirBuilder for DebugConstraintBuilder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type EF = SC::Challenge;
    type ExprEF = SC::Challenge;
    type VarEF = SC::Challenge;
    type MP = TwoRowMatrixView<'a, SC::Challenge>;

    fn permutation(&self) -> Self::MP {
        self.perm
    }

    fn assert_zero_ext<I>(&mut self, x: I)
    where
        I: Into<Self::ExprEF>,
    {
        assert_eq!(
            x.into(),
            SC::Challenge::zero(),
            "constraints must evaluate to zero"
        );
    }

    fn permutation_randomness(&self) -> &[Self::EF] {
        self.perm_challenges
    }
}

impl<'a, M: Machine<SC::Val>, SC> ValidaAirBuilder for DebugConstraintBuilder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type Machine = M;

    fn machine(&self) -> &Self::Machine {
        self.machine
    }
}
