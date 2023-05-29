//! Posiedon2 STARK Columns

use crate::Config;
use p3_air::AirBuilder;
use p3_field::PrimeField;
use valida_derive::AlignedBorrow;
use valida_util::indices_arr;

/// Columns for Single-Row Poseidon2 STARK
///
/// The columns of the STARK are divided into two parts: state registers and S-BOX registers.
/// Because the matrix multiplications are linear functions, we don't need auxiliary registers for
/// the intermediate values.
///
/// As an example, let's consider a `WIDTH = 3` and `SBOX_DEGREE = 5` instance.
///
/// |  0 |  1 |  2 |         3 |         4 |         5 |         6 |         7 |         8 |
/// |----|----|----|-----------|-----------|-----------|-----------|-----------|-----------|
/// | s0 | s1 | s2 | (s0+r0)^3 | (s1+r1)^3 | (s2+r2)^3 | (s0+r0)^5 | (s1+r1)^5 | (s2+r2)^5 |
///
/// Because the S-BOX is a quintic function but we only have degree 3 constraints, we split the
/// computation into the first degree 3 part and then the second degree 2 part. After this part,
/// we import the right most columns into the matrix multiplication which write to the next state
/// section. Each section has `WIDTH`-many state columns
#[repr(C)]
pub struct Columns<
    T,
    const WIDTH: usize,
    const SBOX_DEGREE: usize,
    const SBOX_REGISTERS: usize,
    const HALF_FULL_ROUNDS: usize,
    const PARTIAL_ROUNDS: usize,
> {
    /// Beginning Full Rounds
    pub beginning_full_rounds: [FullRound<T, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>; HALF_FULL_ROUNDS],

    /// Partial Rounds
    pub partial_rounds: [PartialRound<T, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>; PARTIAL_ROUNDS],

    /// Ending Full Rounds
    pub ending_full_rounds: [FullRound<T, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>; HALF_FULL_ROUNDS],
}

impl<
        T,
        const WIDTH: usize,
        const SBOX_DEGREE: usize,
        const SBOX_REGISTERS: usize,
        const HALF_FULL_ROUNDS: usize,
        const PARTIAL_ROUNDS: usize,
    > Columns<T, WIDTH, SBOX_DEGREE, SBOX_REGISTERS, HALF_FULL_ROUNDS, PARTIAL_ROUNDS>
{
    ///
    #[inline]
    pub fn eval<AB>(
        &self,
        state: &mut [AB::Expr; WIDTH],
        beginning_full_round_constants: &[[AB::Expr; WIDTH]; HALF_FULL_ROUNDS],
        partial_round_constants: &[AB::Expr; PARTIAL_ROUNDS],
        ending_full_round_constants: &[[AB::Expr; WIDTH]; HALF_FULL_ROUNDS],
        builder: &mut AB,
    ) where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        // TODO: Add initial linear layer (matmul_external)
        // matmul_external(state);
        for round in 0..HALF_FULL_ROUNDS {
            self.beginning_full_rounds[round].eval(
                state,
                &beginning_full_round_constants[round],
                builder,
            );
        }
        for round in 0..PARTIAL_ROUNDS {
            self.partial_rounds[round].eval(state, &partial_round_constants[round], builder);
        }
        for round in 0..HALF_FULL_ROUNDS {
            self.ending_full_rounds[round].eval(
                state,
                &ending_full_round_constants[round],
                builder,
            );
        }
    }
}

/// Full Round Columns
#[repr(C)]
pub struct FullRound<T, const WIDTH: usize, const SBOX_DEGREE: usize, const SBOX_REGISTERS: usize> {
    /// S-BOX Columns
    pub sbox: [SBox<T, SBOX_DEGREE, SBOX_REGISTERS>; WIDTH],
}

impl<T, const WIDTH: usize, const SBOX_DEGREE: usize, const SBOX_REGISTERS: usize>
    FullRound<T, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>
{
    ///
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
        // TODO: add matrix multiply
        // matmul_external(state);
        todo!()
    }
}

/// Partial Round Columns
#[repr(C)]
pub struct PartialRound<
    T,
    const WIDTH: usize,
    const SBOX_DEGREE: usize,
    const SBOX_REGISTERS: usize,
> {
    /// S-BOX Columns
    pub sbox: SBox<T, SBOX_DEGREE, SBOX_REGISTERS>,
}

impl<T, const WIDTH: usize, const SBOX_DEGREE: usize, const SBOX_REGISTERS: usize>
    PartialRound<T, WIDTH, SBOX_DEGREE, SBOX_REGISTERS>
{
    ///
    #[inline]
    pub fn eval<AB>(
        &self,
        state: &mut [AB::Expr; WIDTH],
        round_constant: &AB::Expr,
        builder: &mut AB,
    ) where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        state[0] += round_constant.clone();
        self.sbox.eval(&mut state[0], builder);
        // TODO: add matrix multiply
        todo!()
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

impl<T, const DEGREE: usize, const REGISTERS: usize> SBox<T, DEGREE, REGISTERS> {
    /// Degree-Register Table
    ///
    /// This table encodes the optimal number of S-BOX registers needed for the degree, where the
    /// degree is the index into this table. A zero is placed for entries that are ignored.
    pub const OPTIMAL_REGISTER_COUNT: [usize; 12] = [0, 0, 0, 1, 0, 2, 0, 3, 0, 0, 0, 3];

    /// Evaluates the S-BOX over a degree-`1` expression `x`.
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
    /// efficiency of the registers we try to do three multiplications per round. This is not the
    /// optimal number of multiplications for all possible degrees, but for the S-BOX powers we are
    /// interested in for Poseidon2 (namely `3`, `5`, `7`, and `11`), we get the optimal number
    /// with this algorithm with the following register table:
    ///
    /// | `DEGREE` | `REGISTERS` |
    /// |:--------:|:-----------:|
    /// | `3`      | `1`         |
    /// | `5`      | `2`         |
    /// | `7`      | `3`         |
    /// | `11`     | `3`         |
    ///
    /// We record this table in [`Self::OPTIMAL_REGISTER_COUNT`]. This algorithm does not perform
    /// optimally for all possible degrees but provides a reasonable solution in most cases.
    #[inline]
    pub fn eval<AB>(&self, x: &mut AB::Expr, builder: &mut AB)
    where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        assert_ne!(REGISTERS, 0, "The number of REGISTERS must be positive.");
        assert!(
            Self::is_unknown_or_optimal(),
            "The number of REGISTERS must be optimal for the given DEGREE."
        );
        let x_squared = x.clone() * x.clone();
        let x_cubed = x_squared.clone() * x.clone();
        self.load(0, x_cubed.clone(), builder);
        if REGISTERS == 1 {
            *x = self.0[0].into();
            return;
        }
        if ((DEGREE - (DEGREE % 3)) / 3) % 3 == 0 {
            (1..REGISTERS - 1).for_each(|j| self.load_product(j, &[0, 0, j - 1], builder));
        } else {
            (1..REGISTERS - 1).for_each(|j| self.load_product(j, &[0, j - 1], builder));
        }
        self.load(
            REGISTERS - 1,
            [x_cubed, x.clone(), x_squared][DEGREE % 3].clone()
                * AB::Expr::from(self.0[REGISTERS - 2]),
            builder,
        );
        *x = self.0[REGISTERS - 1].into();
    }

    /// Returns `true` when the optimal `DEGREE` and `REGISTERS` are chosen correctly.
    #[inline]
    const fn is_unknown_or_optimal() -> bool {
        if DEGREE > 11 {
            return true;
        }
        let optimal_count = Self::OPTIMAL_REGISTER_COUNT[DEGREE];
        optimal_count == REGISTERS || optimal_count == 0
    }

    /// Loads `value` into the `i`-th S-BOX register.
    #[inline]
    fn load<AB>(&self, i: usize, value: AB::Expr, builder: &mut AB)
    where
        T: Copy,
        AB: AirBuilder<Var = T>,
    {
        builder.assert_eq(AB::Expr::from(self.0[i]), value);
    }

    /// Loads the product over all `product` indices the into the `i`-th S-BOX register.
    #[inline]
    fn load_product<AB>(&self, i: usize, product: &[usize], builder: &mut AB)
    where
        T: Copy,
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
