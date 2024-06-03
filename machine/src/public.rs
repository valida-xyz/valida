use alloc::slice;
use core::iter::Cloned;
use p3_field::{AbstractExtensionField, AbstractField, ExtensionField, TwoAdicField};
use p3_interpolation;
use p3_matrix::{
    dense::{RowMajorMatrix, RowMajorMatrixView},
    Matrix, MatrixGet, MatrixRowSlices, MatrixRows,
};
use p3_util::log2_strict_usize;

pub trait PublicValues<F, E>: MatrixRowSlices<F> + MatrixGet<F>
where
    F: TwoAdicField,
    E: ExtensionField<F> + TwoAdicField,
{
    fn interpolate(&self, zeta: E, offset: usize) -> Vec<E>
    where
        Self: core::marker::Sized,
    {
        let height = self.height();
        let log_height = log2_strict_usize(height);
        let g = F::two_adic_generator(log_height);
        let shift = g.powers().nth(offset).unwrap();

        p3_interpolation::interpolate_coset::<F, E, _>(self, shift, zeta)
    }
}

impl<F, E> PublicValues<F, E> for RowMajorMatrix<F>
where
    F: TwoAdicField,
    E: ExtensionField<F> + TwoAdicField,
{
}

impl<F, E> PublicValues<F, E> for RowMajorMatrixView<'_, F>
where
    F: TwoAdicField,
    E: ExtensionField<F> + TwoAdicField,
{
}

// In the case that the public values are a vector rather than a matrix,
// we view it as a matrix with a single row repeated as many times as desired.
pub struct PublicRow<F>(pub Vec<F>);
pub struct PublicRowView<'a, F>(pub &'a [F]);

impl<T> Matrix<T> for PublicRow<T> {
    fn width(&self) -> usize {
        self.0.len()
    }
    fn height(&self) -> usize {
        1
    }
}
impl<T> Matrix<T> for PublicRowView<'_, T> {
    fn width(&self) -> usize {
        self.0.len()
    }
    fn height(&self) -> usize {
        1
    }
}

impl<T: Clone> MatrixRows<T> for PublicRow<T> {
    type Row<'a> = Cloned<slice::Iter<'a, T>> where T: 'a, Self: 'a;

    fn row(&self, _r: usize) -> Self::Row<'_> {
        self.0.iter().cloned()
    }
}
impl<T: Clone> MatrixRows<T> for PublicRowView<'_, T> {
    type Row<'a> = Cloned<slice::Iter<'a, T>> where T: 'a, Self: 'a;

    fn row(&self, _r: usize) -> Self::Row<'_> {
        self.0.iter().cloned()
    }
}

impl<T: Clone> MatrixRowSlices<T> for PublicRow<T> {
    fn row_slice(&self, _r: usize) -> &[T] {
        self.0.iter().as_slice()
    }
}
impl<T: Clone> MatrixRowSlices<T> for PublicRowView<'_, T> {
    fn row_slice(&self, _r: usize) -> &[T] {
        self.0.iter().as_slice()
    }
}

impl<T: Clone> MatrixGet<T> for PublicRow<T> {
    fn get(&self, _r: usize, c: usize) -> T {
        self.0[c].clone()
    }
}
impl<T: Clone> MatrixGet<T> for PublicRowView<'_, T> {
    fn get(&self, _r: usize, c: usize) -> T {
        self.0[c].clone()
    }
}

impl<F, E> PublicValues<F, E> for PublicRow<F>
where
    F: TwoAdicField,
    E: ExtensionField<F> + TwoAdicField,
{
    fn interpolate(&self, _zeta: E, _offset: usize) -> Vec<E> {
        self.0.iter().map(|v| E::from_base(v.clone())).collect()
    }
}
impl<F, E> PublicValues<F, E> for PublicRowView<'_, F>
where
    F: TwoAdicField,
    E: ExtensionField<F> + TwoAdicField,
{
    fn interpolate(&self, _zeta: E, _offset: usize) -> Vec<E> {
        self.0.iter().map(|v| E::from_base(v.clone())).collect()
    }
}

pub enum ValidaPublicValues<F> {
    PublicTrace(RowMajorMatrix<F>),
    PublicVector(PublicRow<F>),
}

impl<F> Matrix<F> for ValidaPublicValues<F> {
    fn width(&self) -> usize {
        match self {
            ValidaPublicValues::PublicTrace(mat) => mat.width(),
            ValidaPublicValues::PublicVector(row) => row.width(),
        }
    }
    fn height(&self) -> usize {
        match self {
            ValidaPublicValues::PublicTrace(mat) => mat.height(),
            ValidaPublicValues::PublicVector(row) => row.height(),
        }
    }
}

impl<F: Clone> MatrixRows<F> for ValidaPublicValues<F> {
    type Row<'a> = Cloned<slice::Iter<'a, F>> where F: 'a, Self: 'a;

    fn row(&self, r: usize) -> Self::Row<'_> {
        match self {
            ValidaPublicValues::PublicTrace(mat) => mat.row(r),
            ValidaPublicValues::PublicVector(row) => row.row(r),
        }
    }
}

impl<F: Clone> MatrixGet<F> for ValidaPublicValues<F> {
    fn get(&self, r: usize, c: usize) -> F {
        match self {
            ValidaPublicValues::PublicTrace(mat) => mat.get(r, c),
            ValidaPublicValues::PublicVector(row) => row.get(r, c),
        }
    }
}

impl<F: Clone> MatrixRowSlices<F> for ValidaPublicValues<F> {
    fn row_slice(&self, r: usize) -> &[F] {
        match self {
            ValidaPublicValues::PublicTrace(mat) => mat.row_slice(r),
            ValidaPublicValues::PublicVector(row) => row.row_slice(r),
        }
    }
}

impl<F, E> PublicValues<F, E> for ValidaPublicValues<F>
where
    F: TwoAdicField,
    E: ExtensionField<F> + TwoAdicField,
{
    fn interpolate(&self, zeta: E, offset: usize) -> Vec<E> {
        match self {
            ValidaPublicValues::PublicTrace(mat) => mat.interpolate(zeta, offset),
            ValidaPublicValues::PublicVector(row) => row.interpolate(zeta, offset),
        }
    }
}
