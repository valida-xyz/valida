use crate::Machine;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use itertools::repeat_n;
use p3_air::{AirBuilder, PermutationAirBuilder, VirtualPairCol};
use p3_field::{AbstractExtensionField, AbstractField, ExtensionField, Field, Powers, PrimeField};
use p3_matrix::{dense::RowMajorMatrix, Matrix};

pub trait Chip<M: Machine> {
    /// Generate the main trace for the chip given the provided machine.
    fn generate_trace(&self, machine: &M) -> RowMajorMatrix<M::F>;

    fn local_sends(&self) -> Vec<Interaction<M::F>> {
        vec![]
    }

    fn local_receives(&self) -> Vec<Interaction<M::F>> {
        vec![]
    }

    fn global_sends(&self, _machine: &M) -> Vec<Interaction<M::F>> {
        vec![]
    }

    fn global_receives(&self, _machine: &M) -> Vec<Interaction<M::F>> {
        vec![]
    }

    fn lookup_data<F: AbstractField, EF: AbstractExtensionField<F>>(
        &self,
    ) -> Option<LookupData<F, EF>> {
        None
    }

    fn set_lookup_data<F: AbstractField, EF: AbstractExtensionField<F>>(
        &mut self,
        _lookup_data: LookupData<F, EF>,
    ) {
    }
}

pub struct Interaction<F: AbstractField> {
    pub fields: Vec<VirtualPairCol<F>>,
    pub count: VirtualPairCol<F>,
    pub argument_index: BusArgumentIndex,
}

impl<F: AbstractField> Interaction<F> {
    fn argument_index(&self) -> usize {
        match self.argument_index {
            BusArgumentIndex::Local(i) => i,
            BusArgumentIndex::Global(i) => i,
        }
    }
}

#[derive(Clone)]
pub enum InteractionType {
    LocalSend,
    LocalReceive,
    GlobalSend,
    GlobalReceive,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum BusArgumentIndex {
    Local(usize),
    Global(usize),
}

pub struct LookupData<F: AbstractField, EF: AbstractExtensionField<F>> {
    pub interactions: Vec<(Interaction<F>, InteractionType)>,

    /// A map from bus ID to the indices of the quotient columns sent/received on that bus.
    pub bus_map: BTreeMap<BusArgumentIndex, Vec<ColumnIndex>>,

    /// The final value of the running sum column
    pub cumulative_sum: EF,
}

type ColumnIndex = usize;

/// Generate the permutation trace for a chip with the provided machine.
/// This is called only after `generate_trace` has been called on all chips.
pub fn generate_permutation_trace<F: Field, M: Machine<F = F>, C: Chip<M>>(
    machine: &M,
    chip: &mut C,
    main: &RowMajorMatrix<M::F>,
    random_elements: Vec<M::EF>,
) -> RowMajorMatrix<M::EF> {
    let all_interactions: Vec<_> = chip
        .local_sends()
        .into_iter()
        .zip(repeat_n(InteractionType::LocalSend, usize::MAX))
        .chain(
            chip.local_receives()
                .into_iter()
                .zip(repeat_n(InteractionType::LocalReceive, usize::MAX)),
        )
        .chain(
            chip.global_sends(machine)
                .into_iter()
                .zip(repeat_n(InteractionType::GlobalSend, usize::MAX)),
        )
        .chain(
            chip.global_receives(machine)
                .into_iter()
                .zip(repeat_n(InteractionType::GlobalReceive, usize::MAX)),
        )
        .collect();

    let (alphas, betas) = generate_rlc_elements(&all_interactions, &random_elements);

    // Compute the quotient columns and build a map from bus to quotient column index
    //
    // Row: | q_1 | q_2 | q_3 | ... | q_n | \phi |
    // * q_i = \frac{1}{\alpha^i + \sum_j \beta^j * f_{i,j}}
    // * f_{i,j} is the jth main trace column for the ith interaction
    // * \phi is the running sum
    //
    // Note: We can optimize this by combining several quotient columns into one (the
    // number is subject to a target constraint degree).
    let perm_width = all_interactions.len() + 1;
    let mut perm_values = Vec::with_capacity(main.height() * perm_width);
    let mut bus_map: BTreeMap<BusArgumentIndex, Vec<ColumnIndex>> = BTreeMap::new();
    for main_row in main.rows() {
        let mut row = vec![M::EF::ZERO; perm_width];
        for (n, (interaction, _)) in all_interactions.iter().enumerate() {
            row[n] = reduce_row(main_row, &interaction.fields, alphas[n], &betas);
            bus_map
                .entry(interaction.argument_index)
                .or_insert_with(Vec::new)
                .push(n);
        }
        perm_values.extend(row);
    }
    let perm_values = batch_invert(perm_values);
    let mut perm = RowMajorMatrix::new(perm_values, perm_width);

    // Compute the running sum column
    let mut phi = vec![M::EF::ZERO; perm.height() + 1];
    for (n, (main_row, perm_row)) in main.rows().zip(perm.rows()).enumerate() {
        phi[n + 1] = phi[n];
        for (m, (interaction, interaction_type)) in all_interactions.iter().enumerate() {
            let mult = interaction.count.apply::<M::F, M::F>(&[], main_row);
            let idx = bus_map[&interaction.argument_index][m];
            match interaction_type {
                InteractionType::LocalSend | InteractionType::GlobalSend => {
                    phi[n + 1] += M::EF::from_base(mult) * perm_row[idx];
                }
                InteractionType::LocalReceive | InteractionType::GlobalReceive => {
                    phi[n + 1] -= M::EF::from_base(mult) * perm_row[idx];
                }
            }
        }
    }

    for (n, row) in perm.as_view_mut().rows().enumerate() {
        *row.last_mut().unwrap() = phi[n];
    }

    // Set lookup data in the chip
    chip.set_lookup_data(LookupData {
        interactions: all_interactions,
        bus_map,
        cumulative_sum: phi[phi.len() - 2],
    });

    perm
}

pub fn eval_permutation_constraints<
    F: PrimeField,
    AB: PermutationAirBuilder<F = F, EF = EF>,
    EF: AbstractExtensionField<AB::Expr> + From<AB::Expr>,
    M: Machine,
    C: Chip<M>,
>(
    chip: &C,
    builder: &mut AB,
) {
    let rand_elems = builder.permutation_randomness().to_vec();

    let main = builder.main();
    let main_local: &[AB::Var] = main.row(0);

    let perm = builder.permutation();
    let perm_width = perm.width();
    let perm_local: &[AB::EF] = perm.row(0);
    let perm_next: &[AB::EF] = perm.row(1);

    let phi_local = perm_local[perm_width - 1].clone();
    let phi_next = perm_next[perm_width - 1].clone();

    let lookup_data = &chip.lookup_data::<AB::Expr, EF>().unwrap();

    let (alphas, betas) = generate_rlc_elements(&lookup_data.interactions, &rand_elems);

    let lhs = phi_next - phi_local.clone();
    let mut rhs = EF::from_base(AB::Expr::from(AB::F::ZERO));
    for (m, (interaction, interaction_type)) in lookup_data.interactions.iter().enumerate() {
        let idx = lookup_data.bus_map[&interaction.argument_index][m];

        // Quotient constraints
        let mut rlc = EF::from_base(AB::Expr::from(AB::F::ZERO));
        for (field, beta) in interaction.fields.iter().zip(betas.clone()) {
            let elem: EF = field.apply::<AB::Expr, AB::Var>(&[], main_local).into();
            rlc += beta * elem;
        }
        rlc += alphas[m].clone();
        builder.assert_eq_ext(rlc, perm_local[idx].clone());

        // Build the RHS of the permutation constraint
        let mult: EF = interaction
            .count
            .apply::<AB::Expr, AB::Var>(&[], main_local)
            .into();
        match interaction_type {
            InteractionType::LocalSend | InteractionType::GlobalSend => {
                rhs += mult * perm_local[idx].clone();
            }
            InteractionType::LocalReceive | InteractionType::GlobalReceive => {
                rhs -= mult * perm_local[idx].clone();
            }
        }
    }

    // Running sum constraints
    builder.when_transition().assert_eq_ext(lhs, rhs);
    builder.when_first_row().assert_zero_ext(phi_local);
    builder
        .when_last_row()
        .assert_eq_ext(perm_local[0].clone(), lookup_data.cumulative_sum.clone());
}

fn reduce_row<F: Field, EF: ExtensionField<F>>(
    row: &[F],
    fields: &[VirtualPairCol<F>],
    alpha: EF,
    betas: &Powers<EF>,
) -> EF {
    let mut rlc = EF::ZERO;
    for (columns, beta) in fields.iter().zip(betas.clone()) {
        rlc += beta * columns.apply::<F, F>(&[], row)
    }
    rlc += alpha;
    rlc
}

fn generate_rlc_elements<F: AbstractField, EF: AbstractExtensionField<F>>(
    interactions: &[(Interaction<F>, InteractionType)],
    random_elements: &[EF],
) -> (Vec<EF>, Powers<EF>) {
    let alphas = {
        let powers = Powers {
            base: random_elements[0].clone(),
            current: EF::from_base(F::ONE),
        };
        interactions
            .iter()
            .map(|(interaction, _)| {
                powers
                    .clone()
                    .skip(interaction.argument_index() + 1)
                    .next()
                    .unwrap()
            })
            .collect::<Vec<EF>>()
    };

    let betas = Powers {
        base: random_elements[1].clone(),
        current: EF::from_base(F::ONE),
    };

    (alphas, betas)
}

pub fn batch_invert<F: Field>(values: Vec<F>) -> Vec<F> {
    let mut res = vec![F::ZERO; values.len()];
    let mut prod = F::ONE;
    for (n, value) in values.iter().cloned().enumerate() {
        res[n] = prod;
        prod *= value;
    }
    let mut inv = prod.inverse();
    for (n, value) in values.iter().cloned().rev().enumerate().rev() {
        res[n] *= inv;
        inv *= value;
    }
    res
}

#[macro_export]
macro_rules! instructions {
    ($($t:ident),*) => {
        $(
            #[derive(Default)]
            pub struct $t {}
        )*
    }
}
