use itertools::Itertools;
use p3_field::Field32;
use std::cmp::Ordering;

/// Generate the permutations columns required for the Halo2 lookup argument
/// (modified from Plonky2)
pub fn permuted_cols<F: Field32>(inputs: &[F], table: &[F]) -> (Vec<F>, Vec<F>) {
    let n = inputs.len();

    let sorted_inputs = inputs
        .iter()
        .cloned()
        .sorted_unstable_by_key(|x| x.as_canonical_u32())
        .collect_vec();
    let sorted_table = table
        .iter()
        .sorted_unstable_by_key(|x| x.as_canonical_u32())
        .collect_vec();

    let mut unused_table_inds = Vec::with_capacity(n);
    let mut unused_table_vals = Vec::with_capacity(n);
    let mut permuted_table = vec![F::ZERO; n];
    let mut i = 0;
    let mut j = 0;
    while (j < n) && (i < n) {
        let input_val = sorted_inputs[i].as_canonical_u32();
        let table_val = sorted_table[j].as_canonical_u32();
        match input_val.cmp(&table_val) {
            Ordering::Greater => {
                unused_table_vals.push(sorted_table[j]);
                j += 1;
            }
            Ordering::Less => {
                if let Some(x) = unused_table_vals.pop() {
                    permuted_table[i] = *x;
                } else {
                    unused_table_inds.push(i);
                }
                i += 1;
            }
            Ordering::Equal => {
                permuted_table[i] = *sorted_table[j];
                i += 1;
                j += 1;
            }
        }
    }

    #[allow(clippy::needless_range_loop)] // indexing is just more natural here
    for jj in j..n {
        unused_table_vals.push(sorted_table[jj]);
    }
    for ii in i..n {
        unused_table_inds.push(ii);
    }
    for (ind, val) in unused_table_inds.into_iter().zip_eq(unused_table_vals) {
        permuted_table[ind] = *val;
    }

    (sorted_inputs, permuted_table)
}
