use p3_air::constraint_consumer::ConstraintConsumer;
use p3_air::window::BasicAirWindow;
use p3_field::field::{Field, FieldExtension};
use p3_field::packed::PackedField;

pub(crate) struct FoldingConstraintConsumer<'a, F, FE, P>
where
    F: Field,
    FE: FieldExtension<F>,
    P: PackedField<Scalar = F>,
{
    window: BasicAirWindow<'a, P>,

    /// Random value used to combine multiple constraints into one.
    alpha: FE,

    /// Running sum of constraints that have been emitted so far, scaled by powers of `alpha`.
    constraint_acc: FE,
}

impl<'a, F, FE, P> ConstraintConsumer<P, BasicAirWindow<'a, P>>
    for FoldingConstraintConsumer<'a, F, FE, P>
where
    F: Field,
    FE: FieldExtension<F>,
    P: PackedField<Scalar = F>,
{
    fn window(&self) -> &BasicAirWindow<'a, P> {
        &self.window
    }

    fn assert_zero<I: Into<P>>(&mut self, constraint: I) {
        // TODO: Could be more efficient if there's a packed version of FE. Use FE::Packing?
        for c in constraint.into().as_slice() {
            self.constraint_acc = (self.constraint_acc * self.alpha) + *c;
        }
    }
}

impl<'a, F, FE, P> FoldingConstraintConsumer<'a, F, FE, P>
where
    F: Field,
    FE: FieldExtension<F>,
    P: PackedField<Scalar = F>,
{
    pub fn new(window: BasicAirWindow<'a, P>, alpha: FE) -> Self {
        Self {
            window,
            constraint_acc: FE::ZERO,
            alpha,
        }
    }

    pub fn accumulator(self) -> FE {
        self.constraint_acc
    }
}
