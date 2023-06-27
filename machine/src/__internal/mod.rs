use core::marker::PhantomData;
use p3_air::TwoRowMatrixView;
use p3_air::{Air, AirBuilder};
use p3_field::Field;
use p3_matrix::{dense::RowMajorMatrix, Matrix, MatrixRows};
use p3_mersenne_31::Mersenne31;

pub type DefaultField = Mersenne31;

pub struct ConstraintFolder<'a, F: Field> {
    main: TwoRowMatrixView<'a, F>,
    is_first_row: F,
    is_last_row: F,
    is_transition: F,
    _phantom_f: PhantomData<F>,
}

impl<'a, F> AirBuilder for ConstraintFolder<'a, F>
where
    F: Field,
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

pub struct BasicSymVar {
    pub row_offset: usize,
    pub column: usize,
}

pub fn prove<F, A>(air: &A, trace: RowMajorMatrix<F>)
where
    F: Field,
    A: for<'a> Air<ConstraintFolder<'a, F>>,
{
    if trace.height() == 0 {
        return;
    }

    // Check that constraints are satisfied
    for n in 0..trace.height() - 1 {
        let local = trace.row(n);
        let next = trace.row(n + 1);

        let mut builder = ConstraintFolder {
            main: TwoRowMatrixView {
                local: &local,
                next: &next,
            },
            is_first_row: F::ZERO,
            is_last_row: F::ZERO,
            is_transition: F::ZERO,
            _phantom_f: PhantomData,
        };
        air.eval(&mut builder);
    }
}
