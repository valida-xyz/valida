use crate::{
    chip::{eval_permutation_constraints, ValidaAirBuilder},
    Chip, Machine,
};
use core::marker::PhantomData;
use p3_air::TwoRowMatrixView;
use p3_air::{Air, AirBuilder, PermutationAirBuilder};
use p3_field::{AbstractExtensionField, AbstractField, ExtensionField, Field, PrimeField};
use p3_matrix::{dense::RowMajorMatrix, Matrix, MatrixRows};
use p3_mersenne_31::Mersenne31;

pub type DefaultField = Mersenne31;

pub struct DebugConstraintBuilder<'a, F: Field, EF: ExtensionField<F>, M: Machine> {
    machine: &'a M,
    main: TwoRowMatrixView<'a, F>,
    perm: TwoRowMatrixView<'a, EF>,
    rand_elems: &'a [EF],
    is_first_row: F,
    is_last_row: F,
    is_transition: F,
    _phantom_f: PhantomData<F>,
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
        self.rand_elems
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

pub fn prove<A, M>(machine: &M, air: &A, main: RowMajorMatrix<M::F>, perm: RowMajorMatrix<M::EF>)
where
    M: Machine,
    A: for<'a> Air<DebugConstraintBuilder<'a, M::F, M::EF, M>> + Chip<M>,
{
    if main.height() == 0 {
        return;
    }

    let cumulative_sum = *perm.row(perm.height() - 1).last().unwrap();

    // Check that constraints are satisfied
    for n in 0..main.height() {
        let main_local = main.row(n);
        let main_next = main.row((n + 1) % main.height());

        let perm_local = perm.row(n);
        let perm_next = perm.row((n + 1) % main.height());

        let mut builder = DebugConstraintBuilder {
            machine,
            main: TwoRowMatrixView {
                local: &main_local,
                next: &main_next,
            },
            perm: TwoRowMatrixView {
                local: &perm_local,
                next: &perm_next,
            },
            rand_elems: &[M::EF::TWO; 3], // FIXME: implement
            is_first_row: M::F::ZERO,
            is_last_row: M::F::ZERO,
            is_transition: M::F::ONE,
            _phantom_f: PhantomData,
        };
        if n == 0 {
            builder.is_first_row = M::F::ONE;
        }
        if n == main.height() - 1 {
            builder.is_last_row = M::F::ONE;
            builder.is_transition = M::F::ZERO;
        }

        air.eval(&mut builder);
        eval_permutation_constraints(air, &mut builder, cumulative_sum);
    }
}
