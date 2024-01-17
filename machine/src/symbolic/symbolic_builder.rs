use alloc::vec;
use alloc::vec::Vec;

use crate::config::StarkConfig;
use crate::{Machine, ValidaAirBuilder};
use p3_air::{Air, AirBuilder, PairBuilder, PermutationAirBuilder};
use p3_matrix::dense::RowMajorMatrix;
use p3_util::log2_ceil_usize;
use valida_machine::symbolic::symbolic_expression_ext::SymbolicExpressionExt;
use valida_machine::symbolic::symbolic_variable::Trace;

use crate::symbolic::symbolic_expression::SymbolicExpression;
use crate::symbolic::symbolic_variable::SymbolicVariable;

pub fn get_log_quotient_degree<M, SC, A>(machine: &M, air: &A) -> usize
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
    A: for<'a> Air<SymbolicAirBuilder<'a, M, SC>>,
{
    // We pad to at least degree 2, since a quotient argument doesn't make sense with smaller degrees.
    let constraint_degree = get_max_constraint_degree(machine, air).max(2);

    // The quotient's actual degree is approximately (max_constraint_degree - 1) n,
    // where subtracting 1 comes from division by the zerofier.
    // But we pad it to a power of two so that we can efficiently decompose the quotient.
    log2_ceil_usize(constraint_degree - 1)
}

pub fn get_max_constraint_degree<M, SC, A>(machine: &M, air: &A) -> usize
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
    A: for<'a> Air<SymbolicAirBuilder<'a, M, SC>>,
{
    get_symbolic_constraints(machine, air)
        .iter()
        .map(|c| c.degree_multiple())
        .max()
        .unwrap_or(0)
}

pub fn get_symbolic_constraints<M, SC, A>(machine: &M, air: &A) -> Vec<SymbolicExpression<SC::Val>>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
    A: for<'a> Air<SymbolicAirBuilder<'a, M, SC>>,
{
    let mut builder = SymbolicAirBuilder::new(machine, air.width());
    air.eval(&mut builder);
    builder.constraints()
}

/// An `AirBuilder` for evaluating constraints symbolically, and recording them for later use.
pub struct SymbolicAirBuilder<'a, M: Machine<SC::Val>, SC: StarkConfig> {
    machine: &'a M,
    preprocessed: RowMajorMatrix<SymbolicVariable<SC::Val>>,
    main: RowMajorMatrix<SymbolicVariable<SC::Val>>,
    permutation: RowMajorMatrix<SymbolicVariable<SC::Challenge>>,
    constraints: Vec<SymbolicExpression<SC::Val>>,
}

impl<'a, M: Machine<SC::Val>, SC: StarkConfig> SymbolicAirBuilder<'a, M, SC> {
    pub(crate) fn new(machine: &'a M, width: usize) -> Self {
        // TODO: `width` is for the main trace, what about others?
        Self {
            machine,
            preprocessed: SymbolicVariable::window(Trace::Preprocessed, width),
            main: SymbolicVariable::window(Trace::Main, width),
            permutation: SymbolicVariable::window(Trace::Permutation, width),
            constraints: vec![],
        }
    }

    pub(crate) fn constraints(self) -> Vec<SymbolicExpression<SC::Val>> {
        self.constraints
    }
}

impl<'a, M: Machine<SC::Val>, SC: StarkConfig> AirBuilder for SymbolicAirBuilder<'a, M, SC> {
    type F = SC::Val;
    type Expr = SymbolicExpression<SC::Val>;
    type Var = SymbolicVariable<SC::Val>;
    type M = RowMajorMatrix<Self::Var>;

    fn main(&self) -> Self::M {
        self.main.clone()
    }

    fn is_first_row(&self) -> Self::Expr {
        SymbolicExpression::IsFirstRow
    }

    fn is_last_row(&self) -> Self::Expr {
        SymbolicExpression::IsLastRow
    }

    fn is_transition_window(&self, size: usize) -> Self::Expr {
        if size == 2 {
            SymbolicExpression::IsTransition
        } else {
            panic!("uni-stark only supports a window size of 2")
        }
    }

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
        self.constraints.push(x.into());
    }
}

impl<'a, M: Machine<SC::Val>, SC: StarkConfig> PairBuilder for SymbolicAirBuilder<'a, M, SC> {
    fn preprocessed(&self) -> Self::M {
        self.preprocessed.clone()
    }
}

impl<'a, M: Machine<SC::Val>, SC: StarkConfig> PermutationAirBuilder
    for SymbolicAirBuilder<'a, M, SC>
{
    type EF = SC::Challenge;
    type ExprEF = SymbolicExpressionExt<SC::Challenge>;
    type VarEF = SymbolicVariable<SC::Challenge>;
    type MP = RowMajorMatrix<Self::VarEF>;

    fn permutation(&self) -> Self::MP {
        self.permutation.clone()
    }

    fn permutation_randomness(&self) -> &[Self::EF] {
        &[] // TODO
    }
}

impl<'a, M: Machine<SC::Val>, SC: StarkConfig> ValidaAirBuilder for SymbolicAirBuilder<'a, M, SC> {
    type Machine = M;

    fn machine(&self) -> &Self::Machine {
        self.machine
    }
}
