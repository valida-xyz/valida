use crate::Machine;
use crate::__internal::ConstraintFolder;
use alloc::vec;
use alloc::vec::Vec;
use valida_util::batch_multiplicative_inverse;

use p3_air::{Air, AirBuilder, PairBuilder, PermutationAirBuilder, VirtualPairCol};
use p3_field::{AbstractExtensionField, AbstractField, ExtensionField, Field, Powers, PrimeField};
use p3_matrix::{dense::RowMajorMatrix, Matrix, MatrixRowSlices};

pub trait Chip<M: Machine>: for<'a> Air<ConstraintFolder<'a, M::F, M::EF, M>> {
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

    fn all_interactions(&self, machine: &M) -> Vec<(Interaction<M::F>, InteractionType)> {
        let mut interactions: Vec<(Interaction<M::F>, InteractionType)> = vec![];
        interactions.extend(
            self.local_sends()
                .into_iter()
                .map(|i| (i, InteractionType::LocalSend)),
        );
        interactions.extend(
            self.local_receives()
                .into_iter()
                .map(|i| (i, InteractionType::LocalReceive)),
        );
        interactions.extend(
            self.global_sends(machine)
                .into_iter()
                .map(|i| (i, InteractionType::GlobalSend)),
        );
        interactions.extend(
            self.global_receives(machine)
                .into_iter()
                .map(|i| (i, InteractionType::GlobalReceive)),
        );
        interactions
    }
}

pub trait ValidaAirBuilder: PairBuilder + PermutationAirBuilder {
    type Machine;

    fn machine(&self) -> &Self::Machine;
}

pub struct Interaction<F: Field> {
    pub fields: Vec<VirtualPairCol<F>>,
    pub count: VirtualPairCol<F>,
    pub argument_index: BusArgument,
}

#[derive(Clone, Debug)]
pub enum InteractionType {
    LocalSend,
    LocalReceive,
    GlobalSend,
    GlobalReceive,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum BusArgument {
    Local(usize),
    Global(usize),
}

impl<F: Field> Interaction<F> {
    pub fn is_local(&self) -> bool {
        match self.argument_index {
            BusArgument::Local(_) => true,
            BusArgument::Global(_) => false,
        }
    }

    pub fn is_global(&self) -> bool {
        match self.argument_index {
            BusArgument::Local(_) => false,
            BusArgument::Global(_) => true,
        }
    }

    pub fn argument_index(&self) -> usize {
        match self.argument_index {
            BusArgument::Local(i) => i,
            BusArgument::Global(i) => i,
        }
    }
}

/// Generate the permutation trace for a chip with the provided machine.
/// This is called only after `generate_trace` has been called on all chips.
pub fn generate_permutation_trace<F, M>(
    machine: &M,
    chip: &dyn Chip<M>,
    main: &RowMajorMatrix<M::F>,
    random_elements: Vec<M::EF>,
) -> RowMajorMatrix<M::EF>
where
    F: Field,
    M: Machine<F = F>,
{
    let all_interactions = chip.all_interactions(machine);
    let (alphas_local, alphas_global) = generate_rlc_elements(machine, chip, &random_elements);
    let betas = random_elements[2].powers();

    let preprocessed = chip.preprocessed_trace();

    // Compute the reciprocal columns
    //
    // Row: | q_1 | q_2 | q_3 | ... | q_n | \phi |
    // * q_i = \frac{1}{\alpha^i + \sum_j \beta^j * f_{i,j}}
    // * f_{i,j} is the jth main trace column for the ith interaction
    // * \phi is the running sum
    //
    // Note: We can optimize this by combining several reciprocal columns into one (the
    // number is subject to a target constraint degree).
    let perm_width = all_interactions.len() + 1;
    let mut perm_values = Vec::with_capacity(main.height() * perm_width);

    for (n, main_row) in main.rows().enumerate() {
        let mut row = vec![M::EF::ZERO; perm_width];
        for (m, (interaction, _)) in all_interactions.iter().enumerate() {
            let alpha_m = if interaction.is_local() {
                alphas_local[interaction.argument_index()]
            } else {
                alphas_global[interaction.argument_index()]
            };
            let preprocessed_row = if preprocessed.is_some() {
                preprocessed.as_ref().unwrap().row_slice(n)
            } else {
                &[]
            };
            row[m] = reduce_row(
                main_row,
                preprocessed_row,
                &interaction.fields,
                alpha_m,
                betas.clone(),
            );
        }
        perm_values.extend(row);
    }
    let perm_values = batch_multiplicative_inverse(perm_values);
    let mut perm = RowMajorMatrix::new(perm_values, perm_width);

    // Compute the running sum column
    let mut phi = vec![M::EF::ZERO; perm.height()];
    for (n, (main_row, perm_row)) in main.rows().zip(perm.rows()).enumerate() {
        if n > 0 {
            phi[n] = phi[n - 1];
        }
        let preprocessed_row = if preprocessed.is_some() {
            preprocessed.as_ref().unwrap().row_slice(n)
        } else {
            &[]
        };
        for (m, (interaction, interaction_type)) in all_interactions.iter().enumerate() {
            let mult = interaction
                .count
                .apply::<M::F, M::F>(preprocessed_row, main_row);
            match interaction_type {
                InteractionType::LocalSend | InteractionType::GlobalSend => {
                    phi[n] += M::EF::from_base(mult) * perm_row[m];
                }
                InteractionType::LocalReceive | InteractionType::GlobalReceive => {
                    phi[n] -= M::EF::from_base(mult) * perm_row[m];
                }
            }
        }
    }

    for (n, row) in perm.as_view_mut().rows_mut().enumerate() {
        *row.last_mut().unwrap() = phi[n];
    }

    perm
}

pub fn eval_permutation_constraints<F, M, C, AB>(chip: &C, builder: &mut AB, cumulative_sum: AB::EF)
where
    F: PrimeField,
    M: Machine<F = F>,
    C: Chip<M> + Air<AB>,
    AB: ValidaAirBuilder<Machine = M, F = F>,
{
    let rand_elems = builder.permutation_randomness().to_vec();

    let main = builder.main();
    let main_local: &[AB::Var] = main.row_slice(0);
    let main_next: &[AB::Var] = main.row_slice(1);

    let preprocessed = builder.preprocessed();
    let preprocessed_local = preprocessed.row_slice(0);
    let preprocessed_next = preprocessed.row_slice(1);

    let perm = builder.permutation();
    let perm_width = perm.width();
    let perm_local: &[AB::VarEF] = perm.row_slice(0);
    let perm_next: &[AB::VarEF] = perm.row_slice(1);

    let phi_local = perm_local[perm_width - 1].clone();
    let phi_next = perm_next[perm_width - 1].clone();

    let all_interactions = chip.all_interactions(builder.machine());

    let (alphas_local, alphas_global) = generate_rlc_elements(builder.machine(), chip, &rand_elems);
    let betas = rand_elems[2].powers();

    let lhs = phi_next - phi_local.clone();
    let mut rhs = AB::ExprEF::from_base(AB::Expr::ZERO);
    let mut phi_0 = AB::ExprEF::from_base(AB::Expr::ZERO);
    for (m, (interaction, interaction_type)) in all_interactions.iter().enumerate() {
        // Reciprocal constraints
        let mut rlc = AB::ExprEF::from_base(AB::Expr::ZERO);
        for (field, beta) in interaction.fields.iter().zip(betas.clone()) {
            let elem = field.apply::<AB::Expr, AB::Var>(preprocessed_local, main_local);
            rlc += AB::ExprEF::from(beta) * elem;
        }
        if interaction.is_local() {
            rlc = rlc + alphas_local[interaction.argument_index()];
        } else {
            rlc = rlc + alphas_global[interaction.argument_index()];
        }
        builder.assert_one_ext::<AB::ExprEF, AB::ExprEF>(rlc * perm_local[m]);

        let mult_local = interaction
            .count
            .apply::<AB::Expr, AB::Var>(preprocessed_local, main_local);
        let mult_next = interaction
            .count
            .apply::<AB::Expr, AB::Var>(preprocessed_next, main_next);

        // Build the RHS of the permutation constraint
        match interaction_type {
            InteractionType::LocalSend | InteractionType::GlobalSend => {
                phi_0 += AB::ExprEF::from_base(mult_local) * perm_local[m];
                rhs += AB::ExprEF::from_base(mult_next) * perm_next[m];
            }
            InteractionType::LocalReceive | InteractionType::GlobalReceive => {
                phi_0 -= AB::ExprEF::from_base(mult_local) * perm_local[m];
                rhs -= AB::ExprEF::from_base(mult_next) * perm_next[m];
            }
        }
    }

    // Running sum constraints
    builder
        .when_transition()
        .assert_eq_ext::<AB::ExprEF, _, _>(lhs, rhs);
    builder
        .when_first_row()
        .assert_eq_ext(perm_local.last().unwrap().clone(), phi_0);
    builder.when_last_row().assert_eq_ext(
        perm_local.last().unwrap().clone(),
        AB::ExprEF::from(cumulative_sum),
    );
}

fn generate_rlc_elements<F, EF, M>(
    machine: &M,
    chip: &dyn Chip<M>,
    random_elements: &[EF],
) -> (Vec<EF>, Vec<EF>)
where
    F: AbstractField,
    EF: AbstractExtensionField<F>,
    M: Machine,
{
    let alphas_local = random_elements[0]
        .powers()
        .skip(1)
        .take(
            chip.local_sends()
                .into_iter()
                .chain(chip.local_receives())
                .into_iter()
                .map(|interaction| interaction.argument_index())
                .max()
                .unwrap_or(0)
                + 1,
        )
        .collect::<Vec<_>>();

    let alphas_global = random_elements[1]
        .powers()
        .skip(1)
        .take(
            chip.global_sends(machine)
                .into_iter()
                .chain(chip.global_receives(machine))
                .into_iter()
                .map(|interaction| interaction.argument_index())
                .max()
                .unwrap_or(0)
                + 1,
        )
        .collect::<Vec<_>>();

    (alphas_local, alphas_global)
}

// TODO: Use Var and Expr type bounds in place of concrete fields so that
// this function can be used in `eval_permutation_constraints`.
fn reduce_row<F, EF>(
    main_row: &[F],
    preprocessed_row: &[F],
    fields: &[VirtualPairCol<F>],
    alpha: EF,
    betas: Powers<EF>,
) -> EF
where
    F: Field,
    EF: ExtensionField<F>,
{
    let mut rlc = EF::ZERO;
    for (columns, beta) in fields.iter().zip(betas) {
        rlc += beta * columns.apply::<F, F>(preprocessed_row, main_row)
    }
    rlc += alpha;
    rlc
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
