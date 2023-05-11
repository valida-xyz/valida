use crate::config::StarkConfig;
use p3_air::Air;
use p3_air::TwoRowMatrixView;
use p3_field::SymbolicField;
use p3_field::{AbstractField, Field};
use p3_matrix::dense::RowMajorMatrix;
use p3_mersenne_31::Mersenne31;
use std::marker::PhantomData;
use std::ops::{Add, Mul, Sub};

pub type DefaultField = Mersenne31;

pub struct BasicFoldingAirBuilder<'a, F, Exp, Var> {
    main: TwoRowMatrixView<'a, Var>,
    is_first_row: Exp,
    is_last_row: Exp,
    is_transition: Exp,
    _phantom_f: PhantomData<F>,
}

pub struct BasicSymVar {
    pub row_offset: usize,
    pub column: usize,
}

pub fn prove<SC, A>(air: &A, trace: RowMajorMatrix<SC::F>)
where
    SC: StarkConfig,
    A: for<'a> Air<
        BasicFoldingAirBuilder<'a, SC::F, <SC::F as Field>::Packing, <SC::F as Field>::Packing>,
    >,
    A: for<'a> Air<
        BasicFoldingAirBuilder<'a, SC::F, SymbolicField<SC::F, BasicSymVar>, BasicSymVar>,
    >,
{
}
