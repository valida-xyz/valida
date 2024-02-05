use crate::{Machine, ValidaAirBuilder};
use p3_air::{AirBuilder, PairBuilder, PermutationAirBuilder, TwoRowMatrixView};
use p3_field::{AbstractField, ExtensionField, Field};
use valida_machine::StarkConfig;

pub struct ProverConstraintFolder<'a, M: Machine<SC::Val>, SC: StarkConfig> {
    pub(crate) machine: &'a M,
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

pub struct VerifierConstraintFolder<'a, M, F, EF> {
    pub(crate) machine: &'a M,
    pub(crate) preprocessed: TwoRowMatrixView<'a, EF>,
    pub(crate) main: TwoRowMatrixView<'a, EF>,
    pub(crate) perm: TwoRowMatrixView<'a, EF>,
    pub(crate) perm_challenges: &'a [EF],
    pub(crate) is_first_row: EF,
    pub(crate) is_last_row: EF,
    pub(crate) is_transition: EF,
    pub(crate) alpha: EF,
    pub(crate) accumulator: EF,
    pub(crate) _phantom: std::marker::PhantomData<F>,
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

impl<'a, M, SC> PermutationAirBuilder for ProverConstraintFolder<'a, M, SC>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
{
    type EF = SC::Challenge;
    type ExprEF = SC::PackedChallenge;
    type VarEF = SC::PackedChallenge;
    type MP = TwoRowMatrixView<'a, SC::PackedChallenge>;

    fn permutation(&self) -> Self::MP {
        self.perm
    }

    fn assert_zero_ext<I>(&mut self, x: I)
    where
        I: Into<Self::ExprEF>,
    {
        let x: SC::PackedChallenge = x.into();
        self.accumulator *= SC::PackedChallenge::from_f(self.alpha);
        self.accumulator += x;
    }

    fn permutation_randomness(&self) -> &[Self::EF] {
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

impl<'a, F, M, EF> AirBuilder for VerifierConstraintFolder<'a, M, F, EF>
where
    F: Field,
    EF: ExtensionField<F>,
    M: Machine<F>,
{
    type F = EF;
    type Expr = EF;
    type Var = EF;
    type M = TwoRowMatrixView<'a, EF>;

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
        let x: EF = x.into();
        self.accumulator *= self.alpha;
        self.accumulator += x;
    }
}

impl<'a, M, F, EF> PermutationAirBuilder for VerifierConstraintFolder<'a, M, F, EF>
where
    F: Field,
    EF: ExtensionField<F>,
    M: Machine<F>,
{
    type EF = EF;

    type ExprEF = EF;

    type VarEF = EF;

    type MP = TwoRowMatrixView<'a, EF>;

    fn permutation(&self) -> Self::MP {
        self.perm
    }

    fn assert_zero_ext<I>(&mut self, x: I)
    where
        I: Into<Self::ExprEF>,
    {
        self.assert_zero(x);
    }

    fn permutation_randomness(&self) -> &[Self::EF] {
        self.perm_challenges
    }
}

impl<'a, M, F, EF> PairBuilder for VerifierConstraintFolder<'a, M, F, EF>
where
    F: Field,
    EF: ExtensionField<F>,
    M: Machine<F>,
{
    fn preprocessed(&self) -> Self::M {
        self.preprocessed
    }
}

impl<'a, M, F, EF> ValidaAirBuilder for VerifierConstraintFolder<'a, M, F, EF>
where
    F: Field,
    EF: ExtensionField<F>,
    M: Machine<F>,
{
    type Machine = M;

    fn machine(&self) -> &Self::Machine {
        self.machine
    }
}
