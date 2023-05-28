//! Posiedon2 STARK Columns

use crate::Config;
use valida_derive::AlignedBorrow;
use valida_util::indices_arr;

/// Poseidon2 Columns
#[repr(C)]
#[derive(AlignedBorrow, Default)]
pub struct Columns<C, T>
where
    C: Config,
{
    ///
    pub sbox: SBox<T>,
}

///
pub struct SBox<T> {}

impl<C, T> Columns<C, T> where C: Config {}

// TODO: Compute these constants
//
// /// Number of Columns
// pub const NUM_COLUMNS = size_of::<Columns<u8>>();
//
// /// Column Indices
// pub const COLUMN_INDICES: Columns<usize> = make_column_map();
//
// /// Builds the column map from the index array.
// #[inline]
// const fn make_column_map<C>() -> Columns<C, usize>
// where
//     C: Config,
// {
//     const NUM_COLUMNS: usize = size_of::<Columns<C, u8>>();
//     let indices = indices_arr::<NUM_COLUMNS>();
//     unsafe { transmute::<[usize; NUM_COLUMNS], Columns<C, usize>>(indices) }
// }
