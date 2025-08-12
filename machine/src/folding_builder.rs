use crate::{Machine, ValidaAirBuilder};
use p3_air::{
    AirBuilder, AirBuilderWithPublicValues, ExtensionBuilder, PairBuilder, PermutationAirBuilder,
    TwoRowMatrixView,
};
use p3_field::AbstractField;
use valida_machine::StarkConfig;

pub struct ProverConstraintFolder<'a, M: Machine<SC::Val>, SC: StarkConfig> {
    pub(crate) machine: &'a M,
    pub(crate) public_values: TwoRowMatrixView<'a, SC::PackedVal>,
    pub(crate) preprocessed: TwoRowMatrixView<'a, SC::PackedVal>,
    pub(crate) main: TwoRowMatrixView<'a, SC::PackedVal>,
    pub(crate) perm: TwoRowMatrixView<'a, SC::PackedChallenge>,
    pub(crate) perm_challenges: &'a [SC::Challenge],
    pub(crate) is_first_row: SC::PackedVal,
    pub(crate) is_last_row: SC::PackedVal,
    pub(crate) is_transition: SC::PackedVal,
    pub(crate) alpha: SC::Challenge,
    pub(crate) accumulator: SC::PackedChallenge,
}

pub struct VerifierConstraintFolder<'a, M, SC: StarkConfig> {
    pub(crate) machine: &'a M,
    pub(crate) preprocessed: TwoRowMatrixView<'a, SC::Challenge>,
    pub(crate) main: TwoRowMatrixView<'a, SC::Challenge>,
    pub(crate) public_values: TwoRowMatrixView<'a, SC::Challenge>,
    pub(crate) perm: TwoRowMatrixView<'a, SC::Challenge>,
    pub(crate) perm_challenges: &'a [SC::Challenge],
    pub(crate) is_first_row: SC::Challenge,
    pub(crate) is_last_row: SC::Challenge,
    pub(crate) is_transition: SC::Challenge,
    pub(crate) alpha: SC::Challenge,
    pub(crate) accumulator: SC::Challenge,
}

impl<'a, M, SC> AirBuilder for ProverConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type F = SC::Val;
    type Expr = SC::PackedVal;
    type Var = SC::PackedVal;
    type M = TwoRowMatrixView<'a, SC::PackedVal>;

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
        let x: SC::PackedVal = x.into();
        self.accumulator *= SC::PackedChallenge::from_f(self.alpha);
        self.accumulator += x;
    }
}

impl<'a, M, SC> PairBuilder for ProverConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    fn preprocessed(&self) -> Self::M {
        self.preprocessed
    }
}

impl<'a, M, SC> ExtensionBuilder for ProverConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type EF = SC::Challenge;
    type ExprEF = SC::PackedChallenge;
    type VarEF = SC::PackedChallenge;

    fn assert_zero_ext<I>(&mut self, x: I)
    where
        I: Into<Self::ExprEF>,
    {
        let x: SC::PackedChallenge = x.into();
        self.accumulator *= SC::PackedChallenge::from_f(self.alpha);
        self.accumulator += x;
    }
}

impl<'a, M, SC> PermutationAirBuilder for ProverConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type MP = TwoRowMatrixView<'a, SC::PackedChallenge>;

    fn permutation(&self) -> Self::MP {
        self.perm
    }

    fn permutation_randomness(&self) -> &[Self::EF] {
        // TODO: implement
        self.perm_challenges
    }
}

impl<'a, M, SC> ValidaAirBuilder for ProverConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type Machine = M;

    fn machine(&self) -> &Self::Machine {
        self.machine
    }
}

impl<'a, M, SC> AirBuilderWithPublicValues for ProverConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    fn public_values(&self) -> Self::M {
        self.public_values
    }
}

impl<'a, M, SC> AirBuilder for VerifierConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type F = SC::Val;
    type Expr = SC::Challenge;
    type Var = SC::Challenge;
    type M = TwoRowMatrixView<'a, SC::Challenge>;

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
            panic!("uni-stark only supports a window size of 2")
        }
    }

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
        let x: SC::Challenge = x.into();
        self.accumulator *= self.alpha;
        self.accumulator += x;
    }
}

impl<'a, M, SC> ExtensionBuilder for VerifierConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type EF = SC::Challenge;
    type ExprEF = SC::Challenge;
    type VarEF = SC::Challenge;

    fn assert_zero_ext<I>(&mut self, x: I)
    where
        I: Into<Self::ExprEF>,
    {
        self.assert_zero(x)
    }
}

impl<'a, M, SC> PermutationAirBuilder for VerifierConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type MP = TwoRowMatrixView<'a, SC::Challenge>;

    fn permutation(&self) -> Self::MP {
        self.perm
    }

    fn permutation_randomness(&self) -> &[Self::EF] {
        self.perm_challenges
    }
}

impl<'a, M, SC> PairBuilder for VerifierConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    fn preprocessed(&self) -> Self::M {
        self.preprocessed
    }
}

impl<'a, M, SC> ValidaAirBuilder for VerifierConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type Machine = M;

    fn machine(&self) -> &Self::Machine {
        self.machine
    }
}

impl<'a, M, SC> AirBuilderWithPublicValues for VerifierConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    fn public_values(&self) -> Self::M {
        self.public_values
    }
}
