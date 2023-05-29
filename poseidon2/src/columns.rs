//! Posiedon2 STARK Columns

use crate::PermutationLinearLayer;
use core::marker::PhantomData;
use p3_air::AirBuilder;
use p3_field::PrimeField;
use valida_derive::AlignedBorrow;
use valida_util::indices_arr;

/// Columns for Single-Row Poseidon2 STARK
///
/// The columns of the STARK are divided into the three different round sections of the Poseidon2
/// Permutation: beginning full rounds, partial rounds, and ending full rounds. For the full
/// rounds we store an [`SBox`] columnset for each state variable, and for the partial rounds we
/// store only for the first state variable. Because the matrix multiplications are linear
/// functions, we need only keep auxiliary columns for the S-BOX computations.
#[repr(C)]
pub struct Columns<
    T,
    L,
    const WIDTH: usize,
    const SBOX_DEGREE: usize,
    const SBOX_REGISTERS: usize,
    const HALF_FULL_ROUNDS: usize,
    const PARTIAL_ROUNDS: usize,
> where
    L: PermutationLinearLayer,
{
    /// Beginning Full Rounds
    pub beginning_full_rounds:
        [FullRound<T, L, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>; HALF_FULL_ROUNDS],

    /// Partial Rounds
    pub partial_rounds: [PartialRound<T, L, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>; PARTIAL_ROUNDS],

    /// Ending Full Rounds
    pub ending_full_rounds: [FullRound<T, L, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>; HALF_FULL_ROUNDS],
}

impl<
        T,
        L,
        const WIDTH: usize,
        const SBOX_DEGREE: usize,
        const SBOX_REGISTERS: usize,
        const HALF_FULL_ROUNDS: usize,
        const PARTIAL_ROUNDS: usize,
    > Columns<T, L, WIDTH, SBOX_DEGREE, SBOX_REGISTERS, HALF_FULL_ROUNDS, PARTIAL_ROUNDS>
where
    L: PermutationLinearLayer,
{
    /// Evaluates all the columns of the Poseidon2 STARK.
    #[inline]
    pub fn eval<AB>(
        &self,
        state: &mut [AB::Expr; WIDTH],
        beginning_full_round_constants: &[[AB::Expr; WIDTH]; HALF_FULL_ROUNDS],
        partial_round_constants: &[AB::Expr; PARTIAL_ROUNDS],
        ending_full_round_constants: &[[AB::Expr; WIDTH]; HALF_FULL_ROUNDS],
        internal_matrix_diagonal: &[AB::Expr; WIDTH],
        builder: &mut AB,
    ) where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        assert_eq!(
            L::WIDTH,
            WIDTH,
            "The WIDTH for this STARK does not match the Linear Layer WIDTH."
        );
        L::matmul_external(state);
        for round in 0..HALF_FULL_ROUNDS {
            self.eval_beginning_full_round(state, round, beginning_full_round_constants, builder);
        }
        for round in 0..PARTIAL_ROUNDS {
            self.eval_partial_round(
                state,
                round,
                partial_round_constants,
                internal_matrix_diagonal,
                builder,
            );
        }
        for round in 0..HALF_FULL_ROUNDS {
            self.eval_ending_full_round(state, round, ending_full_round_constants, builder);
        }
    }

    /// Evaluates a beginning full round with index `round` and `round_constants`.
    #[inline]
    fn eval_beginning_full_round<AB>(
        &self,
        state: &mut [AB::Expr; WIDTH],
        round: usize,
        round_constants: &[AB::Expr; WIDTH],
        builder: &mut AB,
    ) where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        self.beginning_full_rounds[round].eval(state, &round_constants[round], builder);
    }

    /// Evaluates a partial round with index `round`, `round_constants`, and `internal_matrix_diagonal`.
    #[inline]
    fn eval_partial_round<AB>(
        &self,
        state: &mut [AB::Expr; WIDTH],
        round: usize,
        round_constants: &[AB::Expr; WIDTH],
        internal_matrix_diagonal: &[AB::Expr; WIDTH],
        builder: &mut AB,
    ) where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        self.partial_rounds[round].eval(
            state,
            &round_constants[round],
            internal_matrix_diagonal,
            builder,
        );
    }

    /// Evaluates an ending full round with index `round` and `round_constants`.
    #[inline]
    fn eval_ending_full_round<AB>(
        &self,
        state: &mut [AB::Expr; WIDTH],
        round: usize,
        round_constants: &[AB::Expr; WIDTH],
        builder: &mut AB,
    ) where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        self.ending_full_rounds[round].eval(state, &round_constants[round], builder);
    }
}

/// Full Round Columns
#[repr(C)]
pub struct FullRound<
    T,
    L,
    const WIDTH: usize,
    const SBOX_DEGREE: usize,
    const SBOX_REGISTERS: usize,
> where
    L: PermutationLinearLayer,
{
    /// S-BOX Columns
    pub sbox: [SBox<T, SBOX_DEGREE, SBOX_REGISTERS>; WIDTH],

    /// Linear Layer Type Parameter
    __: PhantomData<L>,
}

impl<T, L, const WIDTH: usize, const SBOX_DEGREE: usize, const SBOX_REGISTERS: usize>
    FullRound<T, L, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>
where
    L: PermutationLinearLayer,
{
    /// Evaluates full-round columns of the Poseidon2 STARK.
    #[inline]
    pub fn eval<AB>(
        &self,
        state: &mut [AB::Expr; WIDTH],
        round_constants: &[AB::Expr; WIDTH],
        builder: &mut AB,
    ) where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        for (i, (s, r)) in state.iter_mut().zip(round_constants.iter()).enumerate() {
            *s += r.clone();
            self.sbox[i].eval(s, builder);
        }
        L::matmul_external(state);
    }
}

/// Partial Round Columns
#[repr(C)]
pub struct PartialRound<
    T,
    L,
    const WIDTH: usize,
    const SBOX_DEGREE: usize,
    const SBOX_REGISTERS: usize,
> where
    L: PermutationLinearLayer,
{
    /// S-BOX Columns
    pub sbox: SBox<T, SBOX_DEGREE, SBOX_REGISTERS>,

    /// Linear Layer Type Parameter
    __: PhantomData<L>,
}

impl<T, L, const WIDTH: usize, const SBOX_DEGREE: usize, const SBOX_REGISTERS: usize>
    PartialRound<T, L, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>
where
    L: PermutationLinearLayer,
{
    /// Evaluates partial-round columns of the Poseidon2 STARK.
    #[inline]
    pub fn eval<AB>(
        &self,
        state: &mut [AB::Expr; WIDTH],
        round_constant: &AB::Expr,
        internal_matrix_diagonal: &[AB::Expr; WIDTH],
        builder: &mut AB,
    ) where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        state[0] += round_constant.clone();
        self.sbox.eval(&mut state[0], builder);
        L::matmul_internal(state, internal_matrix_diagonal);
    }
}

/// S-BOX Columns
///
/// Use this column-set for an S-BOX that can be computed in `REGISTERS`-many columns. The S-BOX is
/// checked to ensure that `REGISTERS` is the optimal number of registers for the given `DEGREE`
/// for the degrees given in the Poseidon2 paper: `3`, `5`, `7`, and `11`. See [`Self::eval`] for
/// more information.
#[repr(C)]
pub struct SBox<T, const DEGREE: usize, const REGISTERS: usize>(pub [T; REGISTERS]);

impl<T, const DEGREE: usize, const REGISTERS: usize> SBox<T, DEGREE, REGISTERS>
where
    T: Copy,
{
    /// Optimal Degree-Register Table
    ///
    /// This table encodes the optimal number of S-BOX registers needed for the degree, where the
    /// degree is the index into this table. A zero is placed for entries that are ignored. This
    /// optimality value is asserted by the [`Self::eval`] method because it relies on using this
    /// exact number of registers.
    pub const OPTIMAL_REGISTER_COUNT: [usize; 12] = [0, 0, 0, 1, 0, 2, 0, 3, 0, 0, 0, 3];

    /// Evaluates the S-BOX over a degree-`1` expression `x`.
    ///
    /// # Panics
    ///
    /// This method panics if the number of `REGISTERS` is not chosen optimally for the given
    /// `DEGREE` or if the `DEGREE` is not supported by the S-BOX. The supported degrees are
    /// `3`, `5`, `7`, and `11`.
    ///
    /// # Efficiency Note
    ///
    /// This method computes the S-BOX by computing the cube of `x` and then successively
    /// multiplying the running sum by the cube of `x` until the last multiplication where we use
    /// the appropriate power to reach the final product:
    ///
    /// ```text
    /// (x^3) * (x^3) * ... * (x^k) where k = d mod 3
    /// ```
    ///
    /// The intermediate powers are stored in the auxiliary column registers. To maximize the
    /// efficiency of the registers we try to do three multiplications per round. This algorithm
    /// only multiplies the cube of `x` but a more optimal product would be to find the base-3
    /// decomposition of the `DEGREE` and use that to generate the addition chain. Even this is not
    /// the optimal number of multiplications for all possible degrees, but for the S-BOX powers we
    /// are interested in for Poseidon2 (namely `3`, `5`, `7`, and `11`), we get the optimal number
    /// with this algorithm. We use the following register table:
    ///
    /// | `DEGREE` | `REGISTERS` |
    /// |:--------:|:-----------:|
    /// | `3`      | `1`         |
    /// | `5`      | `2`         |
    /// | `7`      | `3`         |
    /// | `11`     | `3`         |
    ///
    /// We record this table in [`Self::OPTIMAL_REGISTER_COUNT`] and this choice of registers is
    /// enforced by this method.
    #[inline]
    pub fn eval<AB>(&self, x: &mut AB::Expr, builder: &mut AB)
    where
        AB: AirBuilder<Var = T>,
    {
        assert_ne!(REGISTERS, 0, "The number of REGISTERS must be positive.");
        assert!(DEGREE <= 11, "The DEGREE must be less than or equal to 11.");
        assert_eq!(
            REGISTERS,
            Self::OPTIMAL_REGISTER_COUNT[DEGREE],
            "The number of REGISTERS must be optimal for the given DEGREE."
        );
        let x2 = x.clone() * x.clone();
        let x3 = x2.clone() * x.clone();
        self.load(0, x3.clone(), builder);
        if REGISTERS == 1 {
            *x = self.0[0].into();
            return;
        }
        if DEGREE == 11 {
            (1..REGISTERS - 1).for_each(|j| self.load_product(j, &[0, 0, j - 1], builder));
        } else {
            (1..REGISTERS - 1).for_each(|j| self.load_product(j, &[0, j - 1], builder));
        }
        self.load_last_product(x.clone(), x2, x3, builder);
        *x = self.0[REGISTERS - 1].into();
    }

    /// Loads `value` into the `i`-th S-BOX register.
    #[inline]
    fn load<AB>(&self, i: usize, value: AB::Expr, builder: &mut AB)
    where
        AB: AirBuilder<Var = T>,
    {
        builder.assert_eq(AB::Expr::from(self.0[i]), value);
    }

    /// Loads the product over all `product` indices the into the `i`-th S-BOX register.
    #[inline]
    fn load_product<AB>(&self, i: usize, product: &[usize], builder: &mut AB)
    where
        AB: AirBuilder<Var = T>,
    {
        assert!(
            product.len() <= 3,
            "Product is too big. We can only compute at most degree-3 constraints."
        );
        self.load(
            i,
            product.iter().map(|j| AB::Expr::from(self.0[*j])).product(),
            builder,
        );
    }

    /// Loads the final product into the last S-BOX register. The final term in the product is
    /// `pow(x, DEGREE % 3)`.
    #[inline]
    fn load_last_product<AB>(&self, x: AB::Expr, x2: AB::Expr, x3: AB::Expr, builder: &mut AB)
    where
        AB: AirBuilder<Var = T>,
    {
        self.load(
            REGISTERS - 1,
            [x3, x, x2][DEGREE % 3].clone() * AB::Expr::from(self.0[REGISTERS - 2]),
            builder,
        );
    }
}

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
