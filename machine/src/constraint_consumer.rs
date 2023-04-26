use p3_air::constraint_consumer::ConstraintConsumer;
use p3_field::field::{Field, FieldExtension};
use p3_field::packed::PackedField;

pub(crate) struct FoldingConstraintConsumer<F, FE, P>
where
    F: Field,
    FE: FieldExtension<F>,
    P: PackedField<Scalar = F>,
{
    /// Random value used to combine multiple constraints into one.
    alpha: FE,

    /// Running sum of constraints that have been emitted so far, scaled by powers of `alpha`.
    constraint_acc: FE,

    /// The evaluation of `X - g^(n-1)`.
    z_last: P,

    /// The evaluation of the Lagrange basis polynomial which is nonzero at the point associated
    /// with the first trace row, and zero at other points in the subgroup.
    lagrange_basis_first: P,

    /// The evaluation of the Lagrange basis polynomial which is nonzero at the point associated
    /// with the last trace row, and zero at other points in the subgroup.
    lagrange_basis_last: P,
}

impl<F, FE, P> ConstraintConsumer<P> for FoldingConstraintConsumer<F, FE, P>
where
    F: Field,
    FE: FieldExtension<F>,
    P: PackedField<Scalar = F>,
{
    fn assert_zero<I: Into<P>>(&mut self, constraint: I) {
        // TODO: Could be more efficient if there's a packed version of FE. Use FE::Packing?
        for c in constraint.into().as_slice() {
            self.constraint_acc = (self.constraint_acc * self.alpha) + *c;
        }
    }
}

impl<F, FE, P> FoldingConstraintConsumer<F, FE, P>
where
    F: Field,
    FE: FieldExtension<F>,
    P: PackedField<Scalar = F>,
{
    pub fn new(alpha: FE, z_last: P, lagrange_basis_first: P, lagrange_basis_last: P) -> Self {
        Self {
            constraint_acc: FE::ZERO,
            alpha,
            z_last,
            lagrange_basis_first,
            lagrange_basis_last,
        }
    }

    pub fn accumulator(self) -> FE {
        self.constraint_acc
    }
}
