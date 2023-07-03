use crate::__internal::DebugConstraintBuilder;
use crate::chip::eval_permutation_constraints;
use crate::{Chip, Machine};
use core::marker::PhantomData;
use p3_air::{Air, TwoRowMatrixView};
use p3_field::AbstractField;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::{Matrix, MatrixRows};
use p3_maybe_rayon::{MaybeIntoParIter, ParallelIterator};

pub fn evaluate_constraints<A, M>(
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
    (0..main.height()).into_par_iter().for_each(|n| {
        let main_local = main.row(n);
        let main_next = main.row((n + 1) % main.height());

        let perm_local = perm.row(n);
        let perm_next = perm.row((n + 1) % main.height());

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
            rand_elems: &[M::EF::TWO; 3], // FIXME: implement
            is_first_row: M::F::ZERO,
            is_last_row: M::F::ZERO,
            is_transition: M::F::ONE,
            _phantom_f: PhantomData,
        };
        if n == 0 {
            builder.is_first_row = M::F::ONE;
        }
        if n == main.height() - 1 {
            builder.is_last_row = M::F::ONE;
            builder.is_transition = M::F::ZERO;
        }

        air.eval(&mut builder);
        eval_permutation_constraints(air, &mut builder, cumulative_sum);
    });
}
