#![no_std]

extern crate alloc;

use alloc::vec::Vec;

/// Returns `[0, ..., N - 1]`.
pub const fn indices_arr<const N: usize>() -> [usize; N] {
    let mut indices_arr = [0; N];
    let mut i = 0;
    while i < N {
        indices_arr[i] = i;
        i += 1;
    }
    indices_arr
}

/// Tranposes a col-major matrix into a row-major matrix.
pub fn to_rows<const N: usize, F: Copy>(matrix: &[Vec<F>]) -> Vec<[F; N]> {
    let l = matrix[0].len();
    let w = matrix.len();

    let mut transposed: Vec<[F; N]> = Vec::with_capacity(l);
    if w >= l {
        for i in 0..l {
            for j in 0..w {
                transposed[i][j] = matrix[j][i];
            }
        }
    } else {
        for j in 0..w {
            for i in 0..l {
                transposed[i][j] = matrix[j][i];
            }
        }
    }
    transposed
}
