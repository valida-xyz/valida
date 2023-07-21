use core::marker::PhantomData;
use p3_challenger::Challenger;
use p3_commit::MultivariatePCS;
use p3_field::{AbstractExtensionField, ExtensionField, Field, PackedField, PrimeField64};
use p3_matrix::dense::RowMajorMatrix;

pub trait StarkConfig {
    /// The field over which trace data is encoded.
    type Val: PrimeField64;
    /// The field from which the verifier draws random challenges.
    type Challenge: ExtensionField<Self::Val>;

    type PackedChallenge: PackedField<Scalar = Self::Challenge>
        + AbstractExtensionField<<Self::Val as Field>::Packing>;

    /// The polynomial commitment scheme used.
    type PCS: MultivariatePCS<Self::Val, RowMajorMatrix<Self::Val>>;

    /// The `Challenger` (Fiat-Shamir) implementation used.
    type Chal: Challenger<Self::Val>;

    fn pcs(&self) -> &Self::PCS;

    fn challenger(&self) -> Self::Chal;
}

pub struct StarkConfigImpl<Val, Challenge, PackedChallenge, PCS, Chal> {
    pcs: PCS,
    init_challenger: Chal,
    _phantom_val: PhantomData<Val>,
    _phantom_challenge: PhantomData<Challenge>,
    _phantom_packed_challenge: PhantomData<PackedChallenge>,
    _phantom_chal: PhantomData<Chal>,
}

impl<Val, Challenge, PackedChallenge, PCS, Chal>
    StarkConfigImpl<Val, Challenge, PackedChallenge, PCS, Chal>
{
    pub fn new(pcs: PCS, init_challenger: Chal) -> Self {
        Self {
            pcs,
            init_challenger,
            _phantom_val: PhantomData,
            _phantom_challenge: PhantomData,
            _phantom_packed_challenge: PhantomData,
            _phantom_chal: PhantomData,
        }
    }
}

impl<Val, Challenge, PackedChallenge, PCS, Chal> StarkConfig
    for StarkConfigImpl<Val, Challenge, PackedChallenge, PCS, Chal>
where
    Val: PrimeField64,
    Challenge: ExtensionField<Val>,
    PackedChallenge: PackedField<Scalar = Challenge> + AbstractExtensionField<Val::Packing>,
    PCS: MultivariatePCS<Val, RowMajorMatrix<Val>>,
    Chal: Challenger<Val> + Clone,
{
    type Val = Val;
    type Challenge = Challenge;
    type PackedChallenge = PackedChallenge;
    type PCS = PCS;
    type Chal = Chal;

    fn pcs(&self) -> &Self::PCS {
        &self.pcs
    }

    fn challenger(&self) -> Self::Chal {
        self.init_challenger.clone()
    }
}
