use crate::__internal::DebugConstraintBuilder;
use crate::chip::eval_permutation_constraints;
use crate::{Chip, Machine};
use p3_air::{Air, TwoRowMatrixView};
use p3_field::AbstractField;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::{Matrix, MatrixRows};
use p3_maybe_rayon::{MaybeIntoParIter, ParallelIterator};

/// Check that all constraints vanish on the subgroup.
pub fn check_constraints<A, M>(
    machine: &M,
    air: &A,
    main: &RowMajorMatrix<M::F>,
    perm: &RowMajorMatrix<M::EF>,
) where
    M: Machine + Sync,
    A: for<'a> Air<DebugConstraintBuilder<'a, M::F, M::EF, M>> + Chip<M>,
{
    if main.height() == 0 {
        return;
    }

    let cumulative_sum = *perm.row(perm.height() - 1).last().unwrap();

    // Check that constraints are satisfied
    (0..main.height()).into_par_iter().for_each(|i| {
        let i_next = (i + 1) % main.height();

        let main_local = main.row(i);
        let main_next = main.row(i_next);
        let perm_local = perm.row(i);
        let perm_next = perm.row(i_next);

        let mut builder = DebugConstraintBuilder {
            machine,
            main: TwoRowMatrixView {
                local: &main_local,
                next: &main_next,
            },
            perm: TwoRowMatrixView {
                local: &perm_local,
                next: &perm_next,
            },
            perm_challenges: &[M::EF::TWO; 3], // FIXME: implement
            is_first_row: M::F::ZERO,
            is_last_row: M::F::ZERO,
            is_transition: M::F::ONE,
        };
        if i == 0 {
            builder.is_first_row = M::F::ONE;
        }
        if i == main.height() - 1 {
            builder.is_last_row = M::F::ONE;
            builder.is_transition = M::F::ZERO;
        }

        air.eval(&mut builder);
        eval_permutation_constraints(air, &mut builder, cumulative_sum);
    });
}
