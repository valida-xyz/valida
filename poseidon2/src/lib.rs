//! Poseidon2 Chip
//!
//! Implementation of the Poseidon2 Permutation from <https://eprint.iacr.org/2023/323>.

// TODO: Flatten all the Round Constants in the Permutation

#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(missing_docs)]
#![forbid(rustdoc::broken_intra_doc_links)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::{AddAssign, MulAssign};
use p3_matrix::dense::RowMajorMatrix;
use valida_machine::Interaction;

pub mod columns;
pub mod stark;

/// Sealed Trait Module
mod sealed {
    /// Sealed Trait
    pub trait Sealed {}
}

/// Poseidon2 Permutation Linear Layer
pub trait PermutationLinearLayer: sealed::Sealed {
    /// Width of the Permutation
    const WIDTH: usize;

    /// Computes the external matrix multiplication for the Poseidon2 Permutation.
    ///
    /// # Unchecked Lengths
    ///
    /// This function does not check that the length of the `state` slice is equal to the
    /// `WIDTH` constant. This should be checked by the caller.
    fn matmul_external<F>(state: &mut [F])
    where
        F: AddAssign<F> + Clone;

    /// Computes the internal matrix multiplication for the Poseidon2 Permutation.
    ///
    /// # Unchecked Lengths
    ///
    /// This function does not check that the lengths of the `state` slice nor `diagonal` are
    /// equal to the `WIDTH` constant. This should be checked by the caller.
    fn matmul_internal<F>(state: &mut [F], diagonal: &[F])
    where
        F: AddAssign<F> + MulAssign<F> + Clone;
}

/// Poseidon2 Linear Layer
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct LinearLayer<const T: u8>;

impl<const T: u8> sealed::Sealed for LinearLayer<T> {}

impl PermutationLinearLayer for LinearLayer<2> {
    const WIDTH: usize = 2;

    /// Computes the external matrix multiplication for the Poseidon2 Permutation using the
    /// `circ(2, 1)` matrix.
    #[inline]
    fn matmul_external<F>(state: &mut [F])
    where
        F: AddAssign<F> + Clone,
    {
        let mut sum = state[0].clone();
        sum.add_assign(state[1].clone());
        state[0].add_assign(sum.clone());
        state[1].add_assign(sum);
    }

    /// Computes the internal matrix multiplication for the Poseidon2 Permutation using the
    /// fixed `[[2, 1], [1, 3]]` matrix for `WIDTH = 2`.
    #[inline]
    fn matmul_internal<F>(state: &mut [F], _: &[F])
    where
        F: AddAssign<F> + MulAssign<F> + Clone,
    {
        let mut sum = state[0].clone();
        sum.add_assign(state[1].clone());
        state[0].add_assign(sum.clone());
        state[1].add_assign(state[1].clone());
        state[1].add_assign(sum);
    }
}

impl PermutationLinearLayer for LinearLayer<3> {
    const WIDTH: usize = 3;

    /// Computes the external matrix multiplication for the Poseidon2 Permutation using the
    /// `circ(2, 1, 1)` matrix.
    #[inline]
    fn matmul_external<F>(state: &mut [F])
    where
        F: AddAssign<F> + Clone,
    {
        let mut sum = state[0].clone();
        sum.add_assign(state[1].clone());
        sum.add_assign(state[2].clone());
        state[0].add_assign(sum.clone());
        state[1].add_assign(sum.clone());
        state[2].add_assign(sum);
    }

    /// Computes the internal matrix multiplication for the Poseidon2 Permutation using the
    /// fixed `[[2, 1, 1], [1, 2, 1], [1, 1, 3]]` matrix for `WIDTH = 3`.
    #[inline]
    fn matmul_internal<F>(state: &mut [F], _: &[F])
    where
        F: AddAssign<F> + MulAssign<F> + Clone,
    {
        let mut sum = state[0].clone();
        sum.add_assign(state[1].clone());
        sum.add_assign(state[2].clone());
        state[0].add_assign(sum.clone());
        state[1].add_assign(sum.clone());
        state[2].add_assign(state[2].clone());
        state[2].add_assign(sum);
    }
}

/// Defines the [`LinearLayer`] constructions for `WIDTH % 4 == 0` up to `24`.
macro_rules! define_multiple_of_four_width_linear_layers {
    ($($width:literal),+) => {
        $(
            impl PermutationLinearLayer for LinearLayer<$width> {
                const WIDTH: usize = $width;

                // TODO: Add documentation.
                #[inline]
                fn matmul_external<F>(state: &mut [F])
                where
                    F: AddAssign<F> + Clone,
                {
                    let t4 = $width / 4;
                    for i in 0..t4 {
                        let start_index = i * 4;
                        let mut t_0 = state[start_index].clone();
                        t_0.add_assign(state[start_index + 1].clone());
                        let mut t_1 = state[start_index + 2].clone();
                        t_1.add_assign(state[start_index + 3].clone());
                        let mut t_2 = state[start_index + 1].clone();
                        t_2.add_assign(t_2.clone());
                        t_2.add_assign(t_1.clone());
                        let mut t_3 = state[start_index + 3].clone();
                        t_3.add_assign(t_3.clone());
                        t_3.add_assign(t_0.clone());
                        let mut t_4 = t_1.clone();
                        t_4.add_assign(t_4.clone());
                        t_4.add_assign(t_4.clone());
                        t_4.add_assign(t_3.clone());
                        let mut t_5 = t_0.clone();
                        t_5.add_assign(t_5.clone());
                        t_5.add_assign(t_5.clone());
                        t_5.add_assign(t_2.clone());
                        let mut t_6 = t_3.clone();
                        t_6.add_assign(t_5.clone());
                        let mut t_7 = t_2.clone();
                        t_7.add_assign(t_4.clone());
                        state[start_index] = t_6.clone();
                        state[start_index + 1] = t_5.clone();
                        state[start_index + 2] = t_7.clone();
                        state[start_index + 3] = t_4.clone();
                    }
                    let mut stored = state[0..4].to_owned();
                    for l in 0..4 {
                        for j in 1..t4 {
                            stored[l].add_assign(state[4 * j + l].clone());
                        }
                    }
                    for i in 0..state.len() {
                        state[i].add_assign(stored[i % 4].clone());
                    }
                }

                // TODO: Add documentation.
                #[inline]
                fn matmul_internal<F>(state: &mut [F], diagonal: &[F])
                where
                    F: AddAssign<F> + MulAssign<F> + Clone,
                {
                    let mut sum = state[0].clone();
                    state.iter().skip(1).for_each(|s| sum.add_assign(s.clone()));
                    for i in 0..state.len() {
                        state[i].mul_assign(diagonal[i].clone());
                        state[i].add_assign(sum.clone());
                    }
                }
            }
        )+
    };
}

define_multiple_of_four_width_linear_layers!(4, 8, 12, 16, 20, 24);

/// Poseidon2 Permutation S-BOX
pub trait PermutationSBox: sealed::Sealed {
    /// Degree of the Permutation S-BOX
    const SBOX_DEGREE: usize;

    /// Computes the power of `x` to the `SBOX_DEGREE`.
    fn sbox_pow<F>(x: &mut F)
    where
        F: MulAssign<F> + Clone;

    /// Applies the S-BOX power to each element in `state`.
    #[inline]
    fn apply_sbox<F>(state: &mut [F])
    where
        F: MulAssign<F> + Clone,
    {
        state.iter_mut().for_each(Self::sbox_pow);
    }
}

/// Poseidon2 S-BOX
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct SBox<const T: u8>;

impl<const T: u8> sealed::Sealed for SBox<T> {}

impl PermutationSBox for SBox<3> {
    const SBOX_DEGREE: usize = 3;

    #[inline]
    fn sbox_pow<F>(x: &mut F)
    where
        F: MulAssign<F> + Clone,
    {
        let mut x2 = x.clone();
        x2.mul_assign(x.clone());
        x.mul_assign(x2);
    }
}

impl PermutationSBox for SBox<5> {
    const SBOX_DEGREE: usize = 5;

    #[inline]
    fn sbox_pow<F>(x: &mut F)
    where
        F: MulAssign<F> + Clone,
    {
        let mut x4 = x.clone();
        x4.mul_assign(x4.clone());
        x4.mul_assign(x4.clone());
        x.mul_assign(x4);
    }
}

impl PermutationSBox for SBox<7> {
    const SBOX_DEGREE: usize = 7;

    #[inline]
    fn sbox_pow<F>(x: &mut F)
    where
        F: MulAssign<F> + Clone,
    {
        let mut x2 = x.clone();
        x2.mul_assign(x.clone());
        let mut x6 = x2.clone();
        x6.mul_assign(x2.clone());
        x6.mul_assign(x2.clone());
        x.mul_assign(x6);
    }
}

impl PermutationSBox for SBox<11> {
    const SBOX_DEGREE: usize = 11;

    #[inline]
    fn sbox_pow<F>(x: &mut F)
    where
        F: MulAssign<F> + Clone,
    {
        let mut x2 = x.clone();
        x2.mul_assign(x.clone());
        let mut x4 = x2.clone();
        x4.mul_assign(x2.clone());
        let mut x5 = x4.clone();
        x5.mul_assign(x.clone());
        let mut x10 = x5.clone();
        x10.mul_assign(x5.clone());
        x.mul_assign(x10);
    }
}

/// Poseidon2 Constants
pub trait Constants {
    /// Permutation Linear Layer
    type LinearLayer: PermutationLinearLayer;

    /// Permutation S-BOX
    type SBox: PermutationSBox;

    /// Number of Full Rounds
    const FULL_ROUNDS: usize;

    /// Number of Partial Rounds
    const PARTIAL_ROUNDS: usize;
}

/// Poseidon2 Parameter Configuration
pub trait Config: Constants {
    /// Field Type
    type Field: AddAssign<Self::Field> + MulAssign<Self::Field> + Clone;
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
    /// we use the `[[2, 1, 1], [1, 2, 1], [1, 1, 3]]` matrix. For those widths we keep an empty
    /// vector for this field.
    pub internal_matrix_diagonal: Vec<C::Field>,
}

impl<C> Permutation<C>
where
    C: Config,
{
    /// Permutation Width
    pub const WIDTH: usize = C::LinearLayer::WIDTH;

    /// S-BOX Degree
    pub const SBOX_DEGREE: usize = C::SBox::SBOX_DEGREE;

    /// Number of Full Rounds
    pub const FULL_ROUNDS: usize = C::FULL_ROUNDS;

    /// Number of Partial Rounds
    pub const PARTIAL_ROUNDS: usize = C::PARTIAL_ROUNDS;

    /// Half the Number of Full Rounds
    pub const HALF_FULL_ROUNDS: usize = C::FULL_ROUNDS / 2;

    /// Total Number of Rounds
    pub const ROUNDS: usize = C::FULL_ROUNDS + C::PARTIAL_ROUNDS;

    /// Builds a new Poseidon2 `Permutation` instance from the given parameters
    /// checking that the lengths of the vectors are correct. See the [`Self::new_unchecked`]
    /// for an unchecked constructor.
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
        if Self::WIDTH == 2 || Self::WIDTH == 3 {
            assert!(internal_matrix_diagonal.is_empty());
        } else {
            assert_eq!(internal_matrix_diagonal.len(), Self::WIDTH);
        }
        Self::new_unchecked(
            beginning_full_round_constants,
            ending_full_round_constants,
            partial_round_constants,
            internal_matrix_diagonal,
        )
    }

    /// Builds a new Poseidon2 `Permutation` instance from the given parameters
    /// without checking the vectors. See the [`Self::new`] method for more a checked
    /// constructor.
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

    /// Computes the Poseidon2 permutation on the given `state`.
    #[inline]
    pub fn permute(&self, state: &mut [C::Field]) {
        assert_eq!(state.len(), Self::WIDTH);
        C::LinearLayer::matmul_external(state);
        for round in 0..Self::HALF_FULL_ROUNDS {
            Self::add_full_round_constants(state, round, &self.beginning_full_round_constants);
            C::SBox::apply_sbox(state);
            C::LinearLayer::matmul_external(state);
        }
        for round in 0..Self::PARTIAL_ROUNDS {
            self.add_partial_round_constant(state, round);
            C::SBox::sbox_pow(&mut state[0]);
            C::LinearLayer::matmul_internal(state, &self.internal_matrix_diagonal);
        }
        for round in 0..Self::HALF_FULL_ROUNDS {
            Self::add_full_round_constants(state, round, &self.ending_full_round_constants);
            C::SBox::apply_sbox(state);
            C::LinearLayer::matmul_external(state);
        }
    }

    /// Adds the `round_constants` at the given `round` to the `state` for full rounds.
    #[inline]
    fn add_full_round_constants(
        state: &mut [C::Field],
        round: usize,
        round_constants: &[C::Field],
    ) {
        let range = round * Self::WIDTH..(round + 1) * Self::WIDTH;
        for (a, b) in state.iter_mut().zip(round_constants[range].iter()) {
            a.add_assign(b.clone());
        }
    }

    /// Adds the round constant at index `round` to the `state` for partial rounds.
    #[inline]
    fn add_partial_round_constant(&self, state: &mut [C::Field], round: usize) {
        state[0].add_assign(self.partial_round_constants[round].clone());
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
