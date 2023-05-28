//! Posiedon2 STARK Columns

use crate::Config;
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
    const SBOX_REGISTERS: usize,
    const HALF_FULL_ROUNDS: usize,
    const PARTIAL_ROUNDS: usize,
> {
    /// Beginning Full Rounds
    pub beginning_full_rounds: [FullRound<T, WIDTH, SBOX_REGISTERS>; HALF_FULL_ROUNDS],

    /// Partial Rounds
    pub partial_rounds: [PartialRound<T, WIDTH, SBOX_REGISTERS>; PARTIAL_ROUNDS],

    /// Ending Full Rounds
    pub ending_full_rounds: [FullRound<T, WIDTH, SBOX_REGISTERS>; HALF_FULL_ROUNDS],
}

impl<
        T,
        const WIDTH: usize,
        const SBOX_REGISTERS: usize,
        const HALF_FULL_ROUNDS: usize,
        const PARTIAL_ROUNDS: usize,
    > Columns<T, WIDTH, SBOX_REGISTERS, HALF_FULL_ROUNDS, PARTIAL_ROUNDS>
{
    #[inline]
    fn eval<AB>(
        &self,
        initial_state: [AB::F; WIDTH],
        beginning_full_round_constants: &[[AB::F; WIDTH]; HALF_FULL_ROUNDS],
        partial_round_constants: &[AB::F; PARTIAL_ROUNDS],
        ending_full_round_constants: &[[AB::F; WIDTH]; HALF_FULL_ROUNDS],
        builder: &mut AB,
    ) -> [AB::F; WIDTH]
    where
        AB: AirBuilder,
        AB::F: PrimeField,
    {
        let mut state = initial_state;
        for round in 0..HALF_FULL_ROUNDS {
            state = beginning_full_rounds[round].eval(
                state,
                &beginning_full_round_constants[round],
                builder,
            );
        }
        for round in 0..PARTIAL_ROUNDS {
            state = partial_rounds[round].eval(state, &partial_round_constants[round], builder);
        }
        for round in 0..HALF_FULL_ROUNDS {
            state =
                ending_full_rounds[round].eval(state, &ending_full_round_constants[round], builder);
        }
        state
    }
}

/// Full Round Columns
#[repr(C)]
pub struct FullRound<T, const WIDTH: usize, const SBOX_REGISTERS: usize> {
    /// State Columns
    pub state: [T; WIDTH],

    /// S-BOX Columns
    pub sbox: [SBox<T, SBOX_REGISTERS>; WIDTH],
}

impl<T, const WIDTH: usize, const SBOX_REGISTERS: usize> FullRound<T, WIDTH, SBOX_REGISTERS> {
    ///
    #[inline]
    fn eval<AB>(
        &self,
        state: &[AB::F; WIDTH],
        round_constants: &[AB::F; WIDTH],
        builder: &mut AB,
    ) -> [AB::F; WIDTH]
    where
        AB: AirBuilder,
        AB::F: PrimeField,
    {
        for i in 0..WIDTH {
            builder.assert_eq(state[0][i], self.state[0][i]);
        }
        for (i, (s, r)) in self.state.iter().zip(round_constants.iter()).enumerate() {
            self.sbox[i].eval(s + r, builder);
        }
        // TODO: add matrix multiply
        todo!()
    }
}

/// Partial Round Columns
#[repr(C)]
pub struct PartialRound<T, const WIDTH: usize, const SBOX_REGISTERS: usize> {
    /// State Columns
    pub state: [T; WIDTH],

    /// S-BOX Columns
    pub sbox: SBox<T, SBOX_REGISTERS>,
}

impl<T, const WIDTH: usize, const SBOX_REGISTERS: usize> PartialRound<T, WIDTH, SBOX_REGISTERS> {
    ///
    #[inline]
    fn eval<AB>(
        &self,
        state: &[AB::F; WIDTH],
        round_constant: &AB::F,
        builder: &mut AB,
    ) -> [AB::F; WIDTH]
    where
        AB: AirBuilder,
        AB::F: PrimeField,
    {
        for i in 0..WIDTH {
            builder.assert_eq(state[0][i], self.state[0][i]);
        }
        self.sbox.eval(self.state[0] + round_constant, builder);
        // TODO: add matrix multiply
        todo!()
    }
}

/// S-BOX Columns
///
/// Use this column-set for an S-BOX that can be computed in `REGISTERS`-many columns.
#[repr(C)]
pub struct SBox<T, const REGISTERS: usize>(pub [T; REGISTERS]);

impl<T, const REGISTERS: usize> SBox<T, REGISTERS> {
    /// Evaluates the S-BOX by multiplying successive squares of the base element `x` into the
    /// running product, starting by cubing `x` and setting the first register to that value and
    /// then squaring `x` and multiplying the previous register by that value and so on.
    ///
    /// # Efficiency Note
    ///
    /// This is not the most efficient use of these registers and for some powers we will use more
    /// registers than necessary. In general we should compute the smallest addition chain for the
    /// given S-BOX power.
    #[inline]
    pub fn eval<AB>(&self, x: &AB::F, builder: &mut AB) -> AB::F
    where
        AB: AirBuilder,
        AB::F: PrimeField,
    {
        builder.assert_eq(self.0[0], cube(x));
        for j in 1..SBOX_REGISTERS {
            builder.assert_eq(self.0[j], self.0[j - 1] * x * x);
        }
        self.0[SBOX_REGISTERS - 1]
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

///
#[inline]
fn cube<F>(x: F) -> F
where
    F: PrimeField,
{
    x * x * x
}
