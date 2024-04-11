use core::marker::PhantomData;
use p3_challenger::{CanObserve, FieldChallenger};
use p3_commit::{Pcs, UnivariatePcsWithLde};
use p3_field::{AbstractExtensionField, ExtensionField, PackedField, PrimeField32, TwoAdicField};
use p3_matrix::dense::RowMajorMatrix;

pub trait StarkConfig {
    /// The field over which trace data is encoded.
    type Val: PrimeField32 + TwoAdicField; // TODO: Relax to Field?
    type PackedVal: PackedField<Scalar = Self::Val>;

    /// The field from which most random challenges are drawn.
    type Challenge: ExtensionField<Self::Val> + TwoAdicField;
    type PackedChallenge: AbstractExtensionField<Self::PackedVal, F = Self::Challenge> + Copy;

    /// The PCS used to commit to trace polynomials.
    type Pcs: UnivariatePcsWithLde<
        Self::Val,
        Self::Challenge,
        RowMajorMatrix<Self::Val>,
        Self::Challenger,
    >;

    /// The challenger (Fiat-Shamir) implementation used.
    type Challenger: FieldChallenger<Self::Val>
        + CanObserve<<Self::Pcs as Pcs<Self::Val, RowMajorMatrix<Self::Val>>>::Commitment>;

    fn pcs(&self) -> &Self::Pcs;

    fn challenger(&self) -> Self::Challenger;
}

#[derive(Debug)]
pub struct StarkConfigImpl<Val, Challenge, PackedChallenge, Pcs, Challenger> {
    pcs: Pcs,
    init_challenger: Challenger,
    _phantom: PhantomData<(Val, Challenge, PackedChallenge, Challenger)>,
}

impl<Val, Challenge, PackedChallenge, Pcs, Challenger>
    StarkConfigImpl<Val, Challenge, PackedChallenge, Pcs, Challenger>
{
    pub fn new(pcs: Pcs, init_challenger: Challenger) -> Self {
        Self {
            pcs,
            init_challenger,
            _phantom: PhantomData,
        }
    }
}

impl<Val, Challenge, PackedChallenge, Pcs, Challenger> StarkConfig
    for StarkConfigImpl<Val, Challenge, PackedChallenge, Pcs, Challenger>
where
    Val: PrimeField32 + TwoAdicField, // TODO: Relax to Field?
    Challenge: ExtensionField<Val> + TwoAdicField,
    PackedChallenge: AbstractExtensionField<Val::Packing, F = Challenge> + Copy,
    Pcs: UnivariatePcsWithLde<Val, Challenge, RowMajorMatrix<Val>, Challenger>,
    Challenger: FieldChallenger<Val>
        + Clone
        + CanObserve<<Pcs as p3_commit::Pcs<Val, RowMajorMatrix<Val>>>::Commitment>,
{
    type Val = Val;
    type PackedVal = Val::Packing;
    type Challenge = Challenge;
    type PackedChallenge = PackedChallenge;
    type Pcs = Pcs;
    type Challenger = Challenger;

    fn pcs(&self) -> &Self::Pcs {
        &self.pcs
    }

    fn challenger(&self) -> Self::Challenger {
        self.init_challenger.clone()
    }
}
