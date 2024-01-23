use crate::__internal::ProverConstraintFolder;
use crate::config::StarkConfig;
use crate::symbolic::symbolic_builder::get_log_quotient_degree;
use crate::{Chip, Machine};
use itertools::Itertools;
use p3_air::{Air, TwoRowMatrixView};
use p3_commit::UnivariatePcsWithLde;
use p3_field::{
    cyclic_subgroup_coset_known_order, AbstractExtensionField, AbstractField, Field, PackedField,
    TwoAdicField,
};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::{MatrixGet, MatrixRows};
use p3_maybe_rayon::prelude::*;
use p3_uni_stark::{decompose_and_flatten, ZerofierOnCoset};
use tracing::instrument;

pub fn quotient<M, A, SC, PreprocessedTraceLde, MainTraceLde, PermTraceLde>(
    machine: &M,
    config: &SC,
    air: &A,
    log_degree: usize,
    preprocessed_trace_lde: Option<PreprocessedTraceLde>,
    main_trace_lde: MainTraceLde,
    perm_trace_lde: PermTraceLde,
    perm_challenges: &[SC::Challenge],
    alpha: SC::Challenge,
) -> RowMajorMatrix<SC::Val>
where
    M: Machine<SC::Val>,
    A: Chip<M, SC>,
    SC: StarkConfig,
    PreprocessedTraceLde: MatrixRows<SC::Val> + MatrixGet<SC::Val> + Sync,
    MainTraceLde: MatrixRows<SC::Val> + MatrixGet<SC::Val> + Sync,
    PermTraceLde: MatrixRows<SC::Val> + MatrixGet<SC::Val> + Sync,
{
    let pcs = config.pcs();
    let log_quotient_degree = get_log_quotient_degree::<M, SC, A>(machine, air);

    let log_stride_for_quotient = pcs.log_blowup() - log_quotient_degree;
    let preprocessed_trace_lde_for_quotient =
        preprocessed_trace_lde.map(|lde| lde.vertically_strided(1 << log_stride_for_quotient, 0));
    let main_trace_lde_for_quotient =
        main_trace_lde.vertically_strided(1 << log_stride_for_quotient, 0);
    let perm_trace_lde_for_quotient =
        perm_trace_lde.vertically_strided(1 << log_stride_for_quotient, 0);

    let quotient_values = quotient_values::<M, SC, A, _, _, _>(
        machine,
        config,
        air,
        log_degree,
        log_quotient_degree,
        preprocessed_trace_lde_for_quotient,
        main_trace_lde_for_quotient,
        perm_trace_lde_for_quotient,
        perm_challenges,
        alpha,
    );

    decompose_and_flatten::<SC::Val, SC::Challenge>(
        quotient_values,
        SC::Challenge::from_base(pcs.coset_shift()),
        log_quotient_degree,
    )
}

#[instrument(name = "compute quotient polynomial", skip_all)]
fn quotient_values<M, SC, A, PreprocessedTraceLde, MainTraceLde, PermTraceLde>(
    machine: &M,
    config: &SC,
    air: &A,
    log_degree: usize,
    log_quotient_degree: usize,
    preprocessed_trace_lde: Option<PreprocessedTraceLde>,
    main_trace_lde: MainTraceLde,
    perm_trace_lde: PermTraceLde,
    perm_challenges: &[SC::Challenge],
    alpha: SC::Challenge,
) -> Vec<SC::Challenge>
where
    M: Machine<SC::Val>,
    SC: StarkConfig,
    A: for<'a> Air<ProverConstraintFolder<'a, M, SC>>,
    PreprocessedTraceLde: MatrixRows<SC::Val> + MatrixGet<SC::Val> + Sync,
    MainTraceLde: MatrixRows<SC::Val> + MatrixGet<SC::Val> + Sync,
    PermTraceLde: MatrixRows<SC::Val> + MatrixGet<SC::Val> + Sync,
{
    let degree = 1 << log_degree;
    let log_quotient_size = log_degree + log_quotient_degree;
    let quotient_size = 1 << log_quotient_size;
    let g_subgroup = SC::Val::two_adic_generator(log_degree);
    let g_extended = SC::Val::two_adic_generator(log_quotient_size);
    let subgroup_last = g_subgroup.inverse();
    let coset_shift = config.pcs().coset_shift();
    let next_step = 1 << log_quotient_degree;

    let mut coset: Vec<_> =
        cyclic_subgroup_coset_known_order(g_extended, coset_shift, quotient_size).collect();

    let zerofier_on_coset = ZerofierOnCoset::new(log_degree, log_quotient_degree, coset_shift);

    // Evaluations of L_first(x) = Z_H(x) / (x - 1) on our coset s H.
    let mut lagrange_first_evals = zerofier_on_coset.lagrange_basis_unnormalized(0);
    let mut lagrange_last_evals = zerofier_on_coset.lagrange_basis_unnormalized(degree - 1);

    // We have a few vectors of length `quotient_size`, and we're going to take slices therein of
    // length `WIDTH`. In the edge case where `quotient_size < WIDTH`, we need to pad those vectors
    // in order for the slices to exist. The entries beyond quotient_size will be ignored, so we can
    // just use default values.
    for _ in quotient_size..SC::PackedVal::WIDTH {
        coset.push(SC::Val::default());
        lagrange_first_evals.push(SC::Val::default());
        lagrange_last_evals.push(SC::Val::default());
    }

    (0..quotient_size)
        .into_par_iter()
        .step_by(SC::PackedVal::WIDTH)
        .flat_map_iter(|i_local_start| {
            let wrap = |i| i % quotient_size;
            let i_next_start = wrap(i_local_start + next_step);
            let i_range = i_local_start..i_local_start + SC::PackedVal::WIDTH;

            let x = *SC::PackedVal::from_slice(&coset[i_range.clone()]);
            let is_transition = x - subgroup_last;
            let is_first_row = *SC::PackedVal::from_slice(&lagrange_first_evals[i_range.clone()]);
            let is_last_row = *SC::PackedVal::from_slice(&lagrange_last_evals[i_range]);

            let (preprocessed_local, preprocessed_next): (Vec<_>, Vec<_>) =
                match &preprocessed_trace_lde {
                    Some(lde) => {
                        let local = (0..lde.width())
                            .map(|col| {
                                SC::PackedVal::from_fn(|offset| {
                                    let row = wrap(i_local_start + offset);
                                    lde.get(row, col)
                                })
                            })
                            .collect();
                        let next = (0..lde.width())
                            .map(|col| {
                                SC::PackedVal::from_fn(|offset| {
                                    let row = wrap(i_next_start + offset);
                                    lde.get(row, col)
                                })
                            })
                            .collect();
                        (local, next)
                    }
                    None => (vec![], vec![]),
                };

            let main_local: Vec<_> = (0..main_trace_lde.width())
                .map(|col| {
                    SC::PackedVal::from_fn(|offset| {
                        let row = wrap(i_local_start + offset);
                        main_trace_lde.get(row, col)
                    })
                })
                .collect();
            let main_next: Vec<_> = (0..main_trace_lde.width())
                .map(|col| {
                    SC::PackedVal::from_fn(|offset| {
                        let row = wrap(i_next_start + offset);
                        main_trace_lde.get(row, col)
                    })
                })
                .collect();

            let ext_degree = <SC::Challenge as AbstractExtensionField<SC::Val>>::D;
            debug_assert_eq!(perm_trace_lde.width() % ext_degree, 0);
            let perm_width_ext = perm_trace_lde.width() / ext_degree;

            let perm_local: Vec<_> = (0..perm_width_ext)
                .map(|ext_col| {
                    SC::PackedChallenge::from_base_fn(|coeff_idx| {
                        SC::PackedVal::from_fn(|offset| {
                            let row = wrap(i_local_start + offset);
                            perm_trace_lde.get(row, ext_col * ext_degree + coeff_idx)
                        })
                    })
                })
                .collect();
            let perm_next: Vec<_> = (0..perm_width_ext)
                .map(|ext_col| {
                    SC::PackedChallenge::from_base_fn(|coeff_idx| {
                        SC::PackedVal::from_fn(|offset| {
                            let row = wrap(i_next_start + offset);
                            perm_trace_lde.get(row, ext_col * ext_degree + coeff_idx)
                        })
                    })
                })
                .collect();

            let accumulator = SC::PackedChallenge::zero();
            let mut folder = ProverConstraintFolder {
                machine,
                preprocessed: TwoRowMatrixView {
                    local: &preprocessed_local,
                    next: &preprocessed_next,
                },
                main: TwoRowMatrixView {
                    local: &main_local,
                    next: &main_next,
                },
                perm: TwoRowMatrixView {
                    local: &perm_local,
                    next: &perm_next,
                },
                perm_challenges,
                is_first_row,
                is_last_row,
                is_transition,
                alpha,
                accumulator,
            };
            air.eval(&mut folder);

            // quotient(x) = constraints(x) / Z_H(x)
            let zerofier_inv: SC::PackedVal = zerofier_on_coset.eval_inverse_packed(i_local_start);
            let quotient = folder.accumulator * zerofier_inv;

            // "Transpose" D packed base coefficients into WIDTH scalar extension coefficients.
            let limit = SC::PackedVal::WIDTH.min(quotient_size);
            (0..limit).map(move |idx_in_packing| {
                let quotient_value = (0..<SC::Challenge as AbstractExtensionField<SC::Val>>::D)
                    .map(|coeff_idx| quotient.as_base_slice()[coeff_idx].as_slice()[idx_in_packing])
                    .collect_vec();
                SC::Challenge::from_base_slice(&quotient_value)
            })
        })
        .collect()
}
