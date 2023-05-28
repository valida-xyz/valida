//! Poseidon2 Chip
//!
//! Implementation of the Poseidon2 Permutation from <https://eprint.iacr.org/2023/323>.

// TODO: Flatten the Round Constants in the Permutation

#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(missing_docs)]

extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use p3_matrix::dense::RowMajorMatrix;
use valida_machine::Interaction;

/// Sealed Trait Module
mod sealed {
    /// Sealed Trait
    pub trait Sealed {}
}

/// Poseidon2 Valid Width Sizes
pub trait Width: sealed::Sealed {
    /// Width of the Permutation
    const WIDTH: usize;

    /// Internal Matrix Diagonal Type
    ///
    /// This type is used by the `matmul_internal` function to define the diagonal of the internal
    /// matrix.
    type InternalMatrixDiagonal<F>;

    /// Computes the external matrix multiplication for the Poseidon2 Permutation.
    ///
    /// # Unchecked Lengths
    ///
    /// This function does not check that the length of the `state` slice is equal to the
    /// `WIDTH` constant. This should be checked by the caller.
    fn matmul_external<F>(state: &mut [F]);

    /// Computes the internal matrix multiplication for the Poseidon2 Permutation.
    ///
    /// # Unchecked Lengths
    ///
    /// This function does not check that the lengths of the `state` slice nor `diagonal` are
    /// equal to the `WIDTH` constant. This should be checked by the caller.
    fn matmul_internal<F>(state: &mut [F], diagonal: &Self::InternalMatrixDiagonal<F>);
}

/// Poseidon2 Width Constant
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct WIDTH<const T: u8>;

impl<const T: u8> sealed::Sealed for WIDTH<T> {}

impl Width for WIDTH<2> {
    const WIDTH: usize = 2;

    /// Fixed Internal Matrix
    ///
    /// For the `WIDTH = 2` case we use the `[[2, 1], [1, 3]]` matrix.
    type InternalMatrixDiagonal<F> = ();

    /// Computes the external matrix multiplication for the Poseidon2 Permutation using the
    /// `circ(2, 1)` matrix.
    #[inline]
    fn matmul_external<F>(state: &mut [F]) {
        // let mut sum = state[0];
        // sum.add_assign(&state[1]);
        // state[0].add_assign(&sum);
        // state[1].add_assign(&sum);
        todo!()
    }

    ///
    #[inline]
    fn matmul_internal<F>(state: &mut [F], _: &Self::InternalMatrixDiagonal<F>) {
        // let mut sum = state[0];
        // sum.add_assign(&state[1]);
        // state[0].add_assign(&sum);
        // state[1].double_in_place();
        // state[1].add_assign(&sum);
        todo!()
    }
}

impl Width for WIDTH<3> {
    const WIDTH: usize = 3;

    /// Fixed Internal Matrix
    ///
    /// For the `WIDTH = 3` case we use the `[[2, 1, 1], [1, 2, 1], [1, 1, 3]]` matrix.
    type InternalMatrixDiagonal<F> = ();

    /// Computes the external matrix multiplication for the Poseidon2 Permutation using the
    /// `circ(2, 1, 1)` matrix.
    #[inline]
    fn matmul_external<F>(state: &mut [F]) {
        // let mut sum = state[0];
        // sum.add_assign(&state[1]);
        // sum.add_assign(&state[2]);
        // state[0].add_assign(&sum);
        // state[1].add_assign(&sum);
        // state[2].add_assign(&sum);
        todo!()
    }

    ///
    #[inline]
    fn matmul_internal<F>(state: &mut [F], _: &Self::InternalMatrixDiagonal<F>) {
        // let mut sum = state[0];
        // sum.add_assign(&state[1]);
        // sum.add_assign(&state[2]);
        // state[0].add_assign(&sum);
        // state[1].add_assign(&sum);
        // state[2].double_in_place();
        // state[2].add_assign(&sum);
        todo!()
    }
}

macro_rules! define_multiple_of_four_widths {
    ($($width:literal),+) => {
        $(
            impl Width for WIDTH<$width> {
                const WIDTH: usize = $width;

                type InternalMatrixDiagonal<F> = [F; $width];

                ///
                #[inline]
                fn matmul_external<F>(state: &mut [F]) {
                    // let t4 = $width / 4;
                    // for i in 0..t4 {
                    //     let start_index = i * 4;
                    //     let mut t_0 = state[start_index];
                    //     t_0.add_assign(&state[start_index + 1]);
                    //     let mut t_1 = state[start_index + 2];
                    //     t_1.add_assign(&state[start_index + 3]);
                    //     let mut t_2 = state[start_index + 1];
                    //     t_2.double_in_place();
                    //     t_2.add_assign(&t_1);
                    //     let mut t_3 = state[start_index + 3];
                    //     t_3.double_in_place();
                    //     t_3.add_assign(&t_0);
                    //     let mut t_4 = t_1;
                    //     t_4.double_in_place();
                    //     t_4.double_in_place();
                    //     t_4.add_assign(&t_3);
                    //     let mut t_5 = t_0;
                    //     t_5.double_in_place();
                    //     t_5.double_in_place();
                    //     t_5.add_assign(&t_2);
                    //     let mut t_6 = t_3;
                    //     t_6.add_assign(&t_5);
                    //     let mut t_7 = t_2;
                    //     t_7.add_assign(&t_4);
                    //     state[start_index] = t_6;
                    //     state[start_index + 1] = t_5;
                    //     state[start_index + 2] = t_7;
                    //     state[start_index + 3] = t_4;
                    // }
                    // let mut stored = [F::zero(); 4];
                    // for l in 0..4 {
                    //     stored[l] = state[l];
                    //     for j in 1..t4 {
                    //         stored[l].add_assign(&state[4 * j + l]);
                    //     }
                    // }
                    // for i in 0..state.len() {
                    //     state[i].add_assign(&stored[i % 4]);
                    // }
                    todo!()
                }

                ///
                #[inline]
                fn matmul_internal<F>(state: &mut [F], diagonal: &Self::InternalMatrixDiagonal<F>) {
                    // let mut sum = state[0];
                    // state.iter().skip(1).for_each(|s| sum.add_assign(s));
                    // for i in 0..state.len() {
                    //     state[i].mul_assign(&diagonal[i]);
                    //     state[i].add_assign(&sum);
                    // }
                    todo!()
                }
            }
        )+
    };
}

define_multiple_of_four_widths!(4, 8, 12, 16, 20, 24);

/// Poseidon2 Valid S-BOX Degrees
pub trait SBoxDegree: sealed::Sealed {
    ///
    const SBOX_DEGREE: usize;

    ///
    fn sbox_pow<F>(value: &mut F);

    ///
    #[inline]
    fn apply_sbox<F>(state: &mut [F]) {
        state.iter_mut().for_each(Self::sbox_pow);
    }
}

/// Poseidon2 S-BOX Degree Constant
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct SBOX_DEGREE<const T: u8>;

impl<const T: u8> sealed::Sealed for SBOX_DEGREE<T> {}

impl SBoxDegree for SBOX_DEGREE<3> {
    const SBOX_DEGREE: usize = 3;

    #[inline]
    fn sbox_pow<F>(value: &mut F) {
        // let mut v2 = *value;
        // v2.square_in_place();
        // value.mul_assign(&v2);
        todo!()
    }
}

impl SBoxDegree for SBOX_DEGREE<5> {
    const SBOX_DEGREE: usize = 5;

    #[inline]
    fn sbox_pow<F>(value: &mut F) {
        // let mut v4 = *value;
        // v4.square_in_place();
        // v4.square_in_place();
        // value.mul_assign(&v4);
        todo!()
    }
}

impl SBoxDegree for SBOX_DEGREE<7> {
    const SBOX_DEGREE: usize = 7;

    #[inline]
    fn sbox_pow<F>(value: &mut F) {
        // let mut v2 = *value;
        // v2.square_in_place();
        // let mut v5 = *v2;
        // v5.square_in_place();
        // v5.mul_assign(value);
        // v5.mul_assign(&v2);
        todo!()
    }
}

impl SBoxDegree for SBOX_DEGREE<11> {
    const SBOX_DEGREE: usize = 11;

    #[inline]
    fn sbox_pow<F>(value: &mut F) {
        todo!()
    }
}

/// Poseidon2 Constants
pub trait Constants {
    /// Width of the Permutation
    type Width: Width;

    /// Degree of the S-BOX
    type SBoxDegree: SBoxDegree;

    /// Number of Full Rounds
    const FULL_ROUNDS: usize;

    /// Number of Partial Rounds
    const PARTIAL_ROUNDS: usize;
}

/// Poseidon2 Parameter Configuration
pub trait Config: Constants {
    /// Field Type
    type Field;
}

/// Poseidon2 Permutation
pub struct Permutation<C>
where
    C: Config,
{
    /// Round Constants used for the Beginning Full Rounds
    pub beginning_full_round_constants: Vec<C::Field>,

    /// Round Constants used for the Ending Full Rounds
    pub ending_full_round_constants: Vec<C::Field>,

    /// Round Constants used for the Partial Rounds
    pub partial_round_constants: Vec<C::Field>,

    /// Internal Matrix Diagonal
    ///
    /// For the `WIDTH = 2` case we use the `[[2, 1], [1, 3]]` matrix and for the `WIDTH = 3` case
    /// we use the `[[2, 1, 1], [1, 2, 1], [1, 1, 3]]` matrix. Any other values will fail
    /// initialization with `Self::new`.
    pub internal_matrix_diagonal: Vec<C::Field>,
}

impl<C> Permutation<C>
where
    C: Config,
{
    ///
    pub const WIDTH: usize = C::Width::WIDTH;

    ///
    pub const SBOX_DEGREE: usize = C::SBoxDegree::SBOX_DEGREE;

    ///
    pub const FULL_ROUNDS: usize = C::FULL_ROUNDS;

    ///
    pub const PARTIAL_ROUNDS: usize = C::PARTIAL_ROUNDS;

    ///
    pub const HALF_FULL_ROUNDS: usize = C::FULL_ROUNDS / 2;

    ///
    pub const ROUNDS: usize = C::FULL_ROUNDS + C::PARTIAL_ROUNDS;

    ///
    #[inline]
    pub fn new(
        beginning_full_round_constants: Vec<C::Field>,
        ending_full_round_constants: Vec<C::Field>,
        partial_round_constants: Vec<C::Field>,
        internal_matrix_diagonal: Vec<C::Field>,
    ) -> Self {
        assert_eq!(
            beginning_full_round_constants.len(),
            Self::HALF_FULL_ROUNDS * Self::WIDTH
        );
        assert_eq!(
            ending_full_round_constants.len(),
            Self::HALF_FULL_ROUNDS * Self::WIDTH
        );
        assert_eq!(partial_round_constants.len(), Self::PARTIAL_ROUNDS);
        assert_eq!(internal_matrix_diagonal.len(), Self::WIDTH);
        Self::new_unchecked(
            beginning_full_round_constants,
            ending_full_round_constants,
            partial_round_constants,
            internal_matrix_diagonal,
        )
    }

    ///
    #[inline]
    pub fn new_unchecked(
        beginning_full_round_constants: Vec<C::Field>,
        ending_full_round_constants: Vec<C::Field>,
        partial_round_constants: Vec<C::Field>,
        internal_matrix_diagonal: Vec<C::Field>,
    ) -> Self {
        Self {
            beginning_full_round_constants,
            ending_full_round_constants,
            partial_round_constants,
            internal_matrix_diagonal,
        }
    }

    ///
    #[inline]
    pub fn permute(&self, state: &mut [C::Field]) {
        assert_eq!(state.len(), Self::WIDTH);
        for round in 0..Self::HALF_FULL_ROUNDS {
            self.add_beginning_full_round_constants(state, round);
            Self::apply_full_sbox(state);
            Self::matmul_external(state);
        }
        for round in 0..Self::PARTIAL_ROUNDS {
            self.add_partial_round_constant(state, round);
            Self::apply_partial_sbox(state);
            self.matmul_internal(state);
        }
        for round in 0..Self::HALF_FULL_ROUNDS {
            self.add_ending_full_round_constants(state, round);
            Self::apply_full_sbox(state);
            Self::matmul_external(state);
        }
    }

    ///
    #[inline]
    fn add_beginning_full_round_constants(&self, state: &mut [C::Field], round: usize) {
        // let range = round * Self::WIDTH..(round + 1) * Self::WIDTH;
        // state
        //     .iter_mut()
        //     .zip(self.beginning_full_round_constants[range].iter())
        //     .for_each(|(a, b)| a.add_assign(b));
        todo!()
    }

    ///
    #[inline]
    fn add_partial_round_constant(&self, state: &mut [C::Field], round: usize) {
        // state[0].add_assign(&self.partial_round_constants[round]);
        todo!()
    }

    ///
    #[inline]
    fn add_ending_full_round_constants(&self, state: &mut [C::Field], round: usize) {
        // let range = round * Self::WIDTH..(round + 1) * Self::WIDTH;
        // state
        //     .iter_mut()
        //     .zip(self.ending_full_round_constants[range].iter())
        //     .for_each(|(a, b)| a.add_assign(b));
        todo!()
    }

    ///
    #[inline]
    fn apply_full_sbox(state: &mut [C::Field]) {
        C::SBoxDegree::apply_sbox(state);
    }

    ///
    #[inline]
    fn apply_partial_sbox(state: &mut [C::Field]) {
        C::SBoxDegree::sbox_pow(&mut state[0]);
    }

    ///
    #[inline]
    fn matmul_external(state: &mut [C::Field]) {
        C::Width::matmul_external(state);
    }

    ///
    #[inline]
    fn matmul_internal(&self, state: &mut [C::Field]) {
        // C::Width::matmul_internal(
        //     state,
        //     C::Width::internal_matrix_diagonal_from_slice(&self.internal_matrix_diagonal),
        // );
        todo!()
    }
}

// TODO: Implement Chip
//
// ///
// #[derive(Default)]
// pub struct Chip {}
//
// impl<M> valida_machine::Chip<M> for Chip {
//     fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F> {
//         todo!()
//     }
//
//     fn local_sends(&self) -> Vec<Interaction<M::F>> {
//         // TODO: Do we need this?
//         vec![]
//     }
//
//     fn local_receives(&self) -> Vec<Interaction<M::F>> {
//         // TODO: Do we need this?
//         vec![]
//     }
//
//     fn global_sends(&self, machine: &M) -> Vec<Interaction<M::F>> {
//         // TODO: Do we need this?
//         vec![]
//     }
//
//     fn global_receives(&self, machine: &M) -> Vec<Interaction<M::F>> {
//         // TODO: Do we need this?
//         vec![]
//     }
// }
