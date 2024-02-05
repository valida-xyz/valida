use p3_air::TwoRowMatrixView;
use p3_field::AbstractExtensionField;
use p3_field::{AbstractField, Field};
use p3_util::reverse_slice_index_bits;

use crate::folding_builder::VerifierConstraintFolder;
use crate::{Chip, Machine, OodEvaluationMismatch, OpenedValues, StarkConfig};

pub fn verify_constraints<M, C, SC>(
    machine: &M,
    chip: &C,
    opened_values: &OpenedValues<SC::Challenge>,
    log_degree: usize,
    g: SC::Val,
    zeta: SC::Challenge,
    alpha: SC::Challenge,
    permutation_challenges: &[SC::Challenge],
) -> Result<(), OodEvaluationMismatch>
where
    M: Machine<SC::Val>,
    C: Chip<M, SC> + ?Sized,
    SC: StarkConfig,
{
    let z_h = zeta.exp_power_of_2(log_degree) - SC::Challenge::one();
    let is_first_row = z_h / (zeta - SC::Val::one());
    let is_last_row = z_h / (zeta - g.inverse());
    let is_transition = zeta - g.inverse();

    let OpenedValues {
        preprocessed_local,
        preprocessed_next,
        trace_local,
        trace_next,
        permutation_local,
        permutation_next,
        quotient_chunks,
    } = opened_values;

    let monomials = (0..SC::Challenge::D)
        .map(SC::Challenge::monomial)
        .collect::<Vec<_>>();

    let embed_alg = |v: &[SC::Challenge]| {
        v.chunks_exact(SC::Challenge::D)
            .map(|chunk| {
                chunk
                    .iter()
                    .zip(monomials.iter())
                    .map(|(x, m)| *x * *m)
                    .sum()
            })
            .collect::<Vec<SC::Challenge>>()
    };

    // Recompute the quotient as extension elements.
    let mut quotient_parts = quotient_chunks
        .chunks_exact(SC::Challenge::D)
        .map(|chunk| {
            chunk
                .iter()
                .zip(monomials.iter())
                .map(|(x, m)| *x * *m)
                .sum()
        })
        .collect::<Vec<SC::Challenge>>();

    let mut folder = VerifierConstraintFolder::<M, SC::Val, SC::Challenge> {
        machine,
        preprocessed: TwoRowMatrixView {
            local: &preprocessed_local,
            next: &preprocessed_next,
        },
        main: TwoRowMatrixView {
            local: &trace_local,
            next: &trace_next,
        },
        perm: TwoRowMatrixView {
            local: &embed_alg(permutation_local),
            next: &embed_alg(permutation_next),
        },
        perm_challenges: permutation_challenges,
        is_first_row,
        is_last_row,
        is_transition,
        alpha,
        accumulator: SC::Challenge::zero(),
        _phantom: core::marker::PhantomData,
    };
    chip.eval(&mut folder);

    reverse_slice_index_bits(&mut quotient_parts);
    let quotient: SC::Challenge = zeta
        .powers()
        .zip(quotient_parts)
        .map(|(weight, part)| part * weight)
        .sum();

    let folded_constraints = folder.accumulator;

    match folded_constraints == z_h * quotient {
        true => Ok(()),
        false => Err(OodEvaluationMismatch),
    }
}
