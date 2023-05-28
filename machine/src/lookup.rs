use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use itertools::Itertools;

use p3_air::{AirBuilder, PermutationAirBuilder};
use p3_field::{AbstractField, Field, PrimeField};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

/// Column lookup type: [(looking, looked, lookup_id)]
/// - Values in the `looking` column are looked up in the `looked` column
/// - `lookup_id` is used to group together multi-column lookups
type Lookup = (usize, usize, usize);

/// A batched version of the univariate logarithmic derivative (LogUp) lookup argument
/// (see https://eprint.iacr.org/2022/1530.pdf)
///
/// - N is the number of lookups
/// - M is the maximum allowed degree for all lookup-related constraints
///
/// In the matrix returned by `build_trace` the first virtual column is the running sum,
/// followed by multiplicity columns, followed by any quotient columns.
pub struct LogUp<const N: usize, const M: usize> {
    lookups: [Lookup; N],
}

// TODO:
// - Generalize this to arbitrary N, and to more than one looked column
// - Use extension field elements for the running sum
// - Variable names used here are a bit of a tongue twister. We should consider renaming them.
impl<const N: usize, const M: usize> LogUp<N, M> {
    pub fn new(lookups: [Lookup; N]) -> Self {
        assert!(N <= 2, "Up to 2 lookups are supported for now");
        assert_eq!(M, 3, "Degree bounds other than 3 are not yet supported");
        assert_eq!(
            lookups
                .iter()
                .map(|(_, _, lookup_id)| lookup_id)
                .unique()
                .count(),
            N,
            "Multi-column lookups are not yet implemented"
        );
        assert_eq!(
            lookups
                .iter()
                .map(|(looking, _, _)| looking)
                .unique()
                .count(),
            N,
            "Duplicate looking columns are not yet implemented"
        );

        Self { lookups }
    }

    pub fn build_trace<F: PrimeField>(
        &self,
        main: &RowMajorMatrix<F>,
        random_elements: Vec<F>,
    ) -> RowMajorMatrix<F> {
        let lookups = self.lookups;

        // Gather all unique looking indices
        let looking_indices = lookups
            .iter()
            .map(|lookup| lookup.0)
            .unique()
            .collect::<Vec<_>>();

        // Gather all unique looked indices
        let looked_indices = lookups
            .iter()
            .map(|lookup| lookup.1)
            .unique()
            .collect::<Vec<_>>();

        // Copy main trace columns corresponding to the looking and looked indices.
        // We assume that {looking indices} ∩ {looked indices} = ∅.
        let mut looking_columns = vec![Vec::with_capacity(main.height()); looking_indices.len()];
        let mut looked_columns = vec![Vec::with_capacity(main.height()); looked_indices.len()];
        for row in main.rows() {
            for (n, idx) in looking_indices.iter().cloned().enumerate() {
                looking_columns[n].push(row[idx]);
            }
            // For each looked column, add a unique random element to the copied values
            for (n, (idx, rand_elem)) in looked_indices
                .iter()
                .cloned()
                .zip(random_elements.iter().cloned())
                .enumerate()
            {
                looked_columns[n].push(row[idx] + rand_elem);
            }
        }

        // Compute new relative column indices
        let (looking_indices_relative, looked_indices_relative) = lookups
            .iter()
            .map(|lookup| {
                (
                    looking_indices
                        .iter()
                        .position(|&idx| idx == lookup.0)
                        .unwrap(),
                    looked_indices
                        .iter()
                        .position(|&idx| idx == lookup.1)
                        .unwrap(),
                )
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();

        // Compute multiplicities after distributing random elements to the looking columns
        let mut multiplicities = vec![vec![F::ZERO; main.height()]; looked_indices.len()];
        for (n, ((looked_idx, looked_column), rand_elem)) in looked_indices
            .into_iter()
            .zip(looked_columns)
            .zip(random_elements)
            .enumerate()
        {
            for (_, idx_1, _) in lookups.into_iter() {
                // If the looked index is in the lookup:
                // - Add the associated random element to all values in the looking column
                // - Count the number of times that looking values appear in the looked column
                if looked_idx == idx_1 {
                    for elem in looking_columns[n].iter_mut() {
                        *elem += rand_elem;
                    }
                    let counts = count_elements(&looking_columns[n], &looked_column);
                    for (a, b) in multiplicities[n].iter_mut().zip(counts) {
                        *a += b;
                    }
                }
            }
        }

        // Invert all lookup elements
        let looking_inv = batch_invert(&looking_columns);
        let looked_inv = batch_invert(&looking_columns);

        // Running sum column
        let mut running_sum = vec![F::ZERO; main.height()];
        for n in 1..(running_sum.len()) {
            running_sum[n] = running_sum[n - 1];
            for idx in looking_indices_relative.iter().cloned() {
                running_sum[n] += looking_inv[idx][n]
            }
            for idx in looked_indices_relative.iter().cloned() {
                running_sum[n] -= looked_inv[idx][n] * multiplicities[idx][n];
            }
        }

        if N == 1 {
            let mut values = vec![F::ZERO; main.height() * 2];
            for (n, row) in values.chunks_mut(2).enumerate() {
                row[0] = running_sum[n];
                row[1] = multiplicities[0][n];
            }
            RowMajorMatrix::new(values, 2)
        } else if N == 2 {
            let mut values = vec![F::ZERO; main.height() * 3];
            for (n, row) in values.chunks_mut(3).enumerate() {
                row[0] = running_sum[n];
                row[1] = multiplicities[0][n];
                row[2] = looking_inv[0][n];
            }
            RowMajorMatrix::new(values, 3)
        } else {
            panic!("Unreachable")
        }
    }

    pub fn build_constraints<AB: PermutationAirBuilder>(&self, builder: &mut AB) {
        let lookups = self.lookups;

        let main = builder.main();
        let main_local = main.row(0);

        let perm = builder.permutation();
        let perm_local = perm.row(0);
        let perm_next = perm.row(1);

        let rand_elems = builder.permutation_randomness().to_vec();

        // Quotient constraints
        if N == 2 {
            // This assumes that the looked columns are the same
            let f_0 = main_local[lookups[0].0];
            let f_1 = main_local[lookups[1].0];
            let q_0 = perm_local[2];
            builder.when_transition().assert_one(q_0 * f_0 * f_1);
        }

        // Running sum constraints
        let mut lhs = perm_next[0] - perm_local[0];
        let mut rhs = AB::Expr::from(AB::F::ZERO);
        let m_0 = perm_local[1];
        let alpha = rand_elems[0].clone();
        if N == 1 {
            let f_0 = main_local[lookups[0].0]; // Looking
            let t_0 = main_local[lookups[0].1]; // Looked

            lhs *= (f_0 + alpha.clone()) * (t_0 + alpha.clone());
            rhs += t_0 + alpha.clone() - m_0 * (f_0 + alpha);
        } else if N == 2 {
            // This assumes that the looked columns are the same
            let q_0 = perm_local[2];
            let t_0 = main_local[lookups[0].1];

            lhs *= t_0 + alpha.clone();
            rhs += m_0 + q_0 * (t_0 + alpha);
        }
        builder.when_transition().assert_eq(lhs, rhs);
        builder.when_first_row().assert_zero(perm_local[0]);
        builder.when_last_row().assert_zero(perm_local[0]);
    }
}

/// Performs batch inversion on a column-major matrix of nonzero field elements
pub fn batch_invert<F: Field>(cols: &[Vec<F>]) -> Vec<Vec<F>> {
    let n_cols = cols.len();
    let n_rows = cols[0].len();
    let mut res = vec![vec![F::ZERO; n_rows]; n_cols];
    let mut prod = F::ONE;
    for n in 0..n_cols {
        for m in 0..n_rows {
            res[n][m] = prod;
            prod *= cols[n][m];
        }
    }

    let mut inv = prod.inverse();
    for n in (0..n_cols).rev() {
        for m in (0..n_rows).rev() {
            res[n][m] *= inv;
            inv *= cols[n][m];
        }
    }

    res
}

fn count_elements<F: PrimeField>(v1: &[F], v2: &[F]) -> Vec<F> {
    let mut map: BTreeMap<F, F> = BTreeMap::new();

    // Count elements in the first vector
    for &item in v1 {
        *map.entry(item).or_insert(F::ZERO) += F::ONE;
    }

    // Construct the final vector
    v2.iter()
        .map(|item| *map.get(item).unwrap_or(&F::ZERO))
        .collect()
}
