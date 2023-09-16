use crate::__internal::DebugConstraintBuilder;
use crate::chip::eval_permutation_constraints;
use crate::{Chip, Machine};
use p3_air::{Air, TwoRowMatrixView};
use p3_field::AbstractField;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_matrix::MatrixRowSlices;
use p3_maybe_rayon::{MaybeIntoParIter, ParallelIterator};

/// Check that all constraints vanish on the subgroup.
pub fn check_constraints<M, A>(
    machine: &M,
    air: &A,
    main: &RowMajorMatrix<M::F>,
    perm: &RowMajorMatrix<M::EF>,
    perm_challenges: &[M::EF],
) where
    M: Machine + Sync,
    A: for<'a> Air<DebugConstraintBuilder<'a, M::F, M::EF, M>> + Chip<M>,
{
    assert_eq!(main.height(), perm.height());
    let height = main.height();
    if height == 0 {
        return;
    }

    let preprocessed = air.preprocessed_trace();

    let cumulative_sum = *perm.row_slice(perm.height() - 1).last().unwrap();

    // Check that constraints are satisfied.
    (0..height).into_par_iter().for_each(|i| {
        let i_next = (i + 1) % height;

        let main_local = main.row_slice(i);
        let main_next = main.row_slice(i_next);
        let preprocessed_local = if preprocessed.is_some() {
            preprocessed.as_ref().unwrap().row_slice(i)
        } else {
            &[]
        };
        let preprocessed_next = if preprocessed.is_some() {
            preprocessed.as_ref().unwrap().row_slice(i_next)
        } else {
            &[]
        };
        let perm_local = perm.row_slice(i);
        let perm_next = perm.row_slice(i_next);

        let mut builder = DebugConstraintBuilder {
            machine,
            main: TwoRowMatrixView {
                local: &main_local,
                next: &main_next,
            },
            preprocessed: TwoRowMatrixView {
                local: &preprocessed_local,
                next: &preprocessed_next,
            },
            perm: TwoRowMatrixView {
                local: &perm_local,
                next: &perm_next,
            },
            perm_challenges,
            is_first_row: M::F::ZERO,
            is_last_row: M::F::ZERO,
            is_transition: M::F::ONE,
        };
        if i == 0 {
            builder.is_first_row = M::F::ONE;
        }
        if i == height - 1 {
            builder.is_last_row = M::F::ONE;
            builder.is_transition = M::F::ZERO;
        }

        air.eval(&mut builder);
        eval_permutation_constraints(air, &mut builder, cumulative_sum);
    });
}

/// Check that the combined cumulative sum across all lookup tables is zero.
pub fn check_cumulative_sums<M>(perms: &[RowMajorMatrix<M::EF>])
where
    M: Machine + Sync,
{
    let sum: M::EF = perms
        .iter()
        .map(|perm| *perm.row_slice(perm.height() - 1).last().unwrap())
        .sum();
    assert_eq!(sum, M::EF::ZERO);
}
