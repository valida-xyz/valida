use core::marker::PhantomData;
use core::ops::{Add, Mul, Sub};

use p3_field::{ExtensionField, Field};
use p3_matrix::dense::RowMajorMatrix;

use crate::symbolic::symbolic_expression::SymbolicExpression;

#[derive(Copy, Clone, Debug)]
pub enum Trace {
    Preprocessed,
    Main,
    Permutation,
    Public,
}

/// A variable within the evaluation window, i.e. a column in either the local or next row.
#[derive(Copy, Clone, Debug)]
pub struct SymbolicVariable<F: Field> {
    pub trace: Trace,
    pub is_next: bool,
    pub column: usize,
    pub(crate) _phantom: PhantomData<F>,
}

impl<F: Field> SymbolicVariable<F> {
    pub(crate) fn window(trace: Trace, width: usize) -> RowMajorMatrix<Self> {
        let values = [false, true]
            .into_iter()
            .flat_map(|is_next| {
                (0..width).map(move |column| SymbolicVariable {
                    trace,
                    is_next,
                    column,
                    _phantom: PhantomData,
                })
            })
            .collect();
        RowMajorMatrix::new(values, width)
    }

    pub(crate) fn to_ext<FE: ExtensionField<F>>(self) -> SymbolicVariable<FE> {
        SymbolicVariable {
            trace: self.trace,
            is_next: self.is_next,
            column: self.column,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn degree_multiple(&self) -> usize {
        match self.trace {
            Trace::Preprocessed => 1,
            Trace::Main => 1,
            Trace::Permutation => 1,
            Trace::Public => 0,
        }
    }
}

impl<F: Field> From<SymbolicVariable<F>> for SymbolicExpression<F> {
    fn from(value: SymbolicVariable<F>) -> Self {
        SymbolicExpression::Variable(value)
    }
}

impl<F: Field> Add for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn add(self, rhs: Self) -> Self::Output {
        SymbolicExpression::from(self) + SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Add<F> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn add(self, rhs: F) -> Self::Output {
        SymbolicExpression::from(self) + SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Add<SymbolicExpression<F>> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn add(self, rhs: SymbolicExpression<F>) -> Self::Output {
        SymbolicExpression::from(self) + rhs
    }
}

impl<F: Field> Add<SymbolicVariable<F>> for SymbolicExpression<F> {
    type Output = Self;

    fn add(self, rhs: SymbolicVariable<F>) -> Self::Output {
        self + Self::from(rhs)
    }
}

impl<F: Field> Sub for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn sub(self, rhs: Self) -> Self::Output {
        SymbolicExpression::from(self) - SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Sub<F> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn sub(self, rhs: F) -> Self::Output {
        SymbolicExpression::from(self) - SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Sub<SymbolicExpression<F>> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn sub(self, rhs: SymbolicExpression<F>) -> Self::Output {
        SymbolicExpression::from(self) - rhs
    }
}

impl<F: Field> Sub<SymbolicVariable<F>> for SymbolicExpression<F> {
    type Output = Self;

    fn sub(self, rhs: SymbolicVariable<F>) -> Self::Output {
        self - Self::from(rhs)
    }
}

impl<F: Field> Mul for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn mul(self, rhs: Self) -> Self::Output {
        SymbolicExpression::from(self) * SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Mul<F> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn mul(self, rhs: F) -> Self::Output {
        SymbolicExpression::from(self) * SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Mul<SymbolicExpression<F>> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn mul(self, rhs: SymbolicExpression<F>) -> Self::Output {
        SymbolicExpression::from(self) * rhs
    }
}

impl<F: Field> Mul<SymbolicVariable<F>> for SymbolicExpression<F> {
    type Output = Self;

    fn mul(self, rhs: SymbolicVariable<F>) -> Self::Output {
        self * Self::from(rhs)
    }
}
