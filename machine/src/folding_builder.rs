use crate::{Machine, ValidaAirBuilder};
use p3_air::{AirBuilder, PairBuilder, PermutationAirBuilder, TwoRowMatrixView};
use p3_field::{AbstractExtensionField, AbstractField, ExtensionField, Field, Res};
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

pub struct VerifierConstraintFolder<'a, M, F, EF, EA> {
    pub(crate) machine: &'a M,
    pub(crate) preprocessed: TwoRowMatrixView<'a, Res<F, EF>>,
    pub(crate) main: TwoRowMatrixView<'a, Res<F, EF>>,
    pub(crate) perm: TwoRowMatrixView<'a, EA>,
    pub(crate) perm_challenges: &'a [EF],
    pub(crate) is_first_row: EF,
    pub(crate) is_last_row: EF,
    pub(crate) is_transition: EF,
    pub(crate) alpha: EF,
    pub(crate) accumulator: Res<F, EF>,
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

impl<'a, M, F, EF, EA> AirBuilder for VerifierConstraintFolder<'a, M, F, EF, EA>
where
    F: Field,
    EF: ExtensionField<F>,
    EA: AbstractExtensionField<Res<F, EF>, F = EF>,
    M: Machine<F>,
{
    type F = F;
    type Expr = Res<F, EF>;
    type Var = Res<F, EF>;
    type M = TwoRowMatrixView<'a, Res<F, EF>>;

    fn main(&self) -> Self::M {
        self.main
    }

    fn is_first_row(&self) -> Self::Expr {
        Res::from_inner(self.is_first_row)
    }

    fn is_last_row(&self) -> Self::Expr {
        Res::from_inner(self.is_last_row)
    }

    fn is_transition_window(&self, size: usize) -> Self::Expr {
        if size == 2 {
            Res::from_inner(self.is_transition)
        } else {
            panic!("uni-stark only supports a window size of 2")
        }
    }

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
        let x: Res<F, EF> = x.into();
        self.accumulator *= Self::Expr::from_inner(self.alpha);
        self.accumulator += x;
    }
}

impl<'a, M, F, EF, EA> PermutationAirBuilder for VerifierConstraintFolder<'a, M, F, EF, EA>
where
    F: Field,
    EF: ExtensionField<F>,
    EA: AbstractExtensionField<Res<F, EF>, F = EF> + Copy,
    M: Machine<F>,
{
    type EF = EF;

    type ExprEF = EA;

    type VarEF = EA;

    type MP = TwoRowMatrixView<'a, EA>;

    fn permutation(&self) -> Self::MP {
        self.perm
    }

    fn permutation_randomness(&self) -> &[Self::EF] {
        self.perm_challenges
    }
}

impl<'a, M, F, EF, EA> PairBuilder for VerifierConstraintFolder<'a, M, F, EF, EA>
where
    F: Field,
    EF: ExtensionField<F>,
    EA: AbstractExtensionField<Res<F, EF>, F = EF> + Copy,
    M: Machine<F>,
{
    fn preprocessed(&self) -> Self::M {
        self.preprocessed
    }
}

impl<'a, M, F, EF, EA> ValidaAirBuilder for VerifierConstraintFolder<'a, M, F, EF, EA>
where
    F: Field,
    EF: ExtensionField<F>,
    EA: AbstractExtensionField<Res<F, EF>, F = EF> + Copy,
    M: Machine<F>,
{
    type Machine = M;

    fn machine(&self) -> &Self::Machine {
        self.machine
    }
}
