use core::marker::PhantomData;
use p3_challenger::FieldChallenger;
use p3_commit::UnivariatePcs;
use p3_dft::TwoAdicSubgroupDft;
use p3_field::{
    AbstractExtensionField, ExtensionField, Field, PackedField, PrimeField64, TwoAdicField,
};
use p3_matrix::dense::RowMajorMatrix;

pub trait StarkConfig {
    /// The field over which trace data is encoded.
    type Val: PrimeField64 + TwoAdicField;
    /// The field from which the verifier draws random challenges.
    type Challenge: ExtensionField<Self::Val>;

    type PackedChallenge: PackedField<Scalar = Self::Challenge>
        + AbstractExtensionField<<Self::Val as Field>::Packing>;

    /// The polynomial commitment scheme used.
    type PCS: UnivariatePcs<Self::Val, RowMajorMatrix<Self::Val>, Self::Chal>;

    type DFT: TwoAdicSubgroupDft<Self::Val>;

    /// The `Challenger` (Fiat-Shamir) implementation used.
    type Chal: FieldChallenger<Self::Val>;

    fn pcs(&self) -> &Self::PCS;

    fn dft(&self) -> &Self::DFT;

    fn challenger(&self) -> Self::Chal;
}

pub struct StarkConfigImpl<Val, Challenge, PackedChallenge, PCS, DFT, Chal> {
    pcs: PCS,
    dft: DFT,
    init_challenger: Chal,
    _phantom_val: PhantomData<Val>,
    _phantom_challenge: PhantomData<Challenge>,
    _phantom_packed_challenge: PhantomData<PackedChallenge>,
    _phantom_chal: PhantomData<Chal>,
}

impl<Val, Challenge, PackedChallenge, PCS, DFT, Chal>
    StarkConfigImpl<Val, Challenge, PackedChallenge, PCS, DFT, Chal>
{
    pub fn new(pcs: PCS, dft: DFT, init_challenger: Chal) -> Self {
        Self {
            pcs,
            dft,
            init_challenger,
            _phantom_val: PhantomData,
            _phantom_challenge: PhantomData,
            _phantom_packed_challenge: PhantomData,
            _phantom_chal: PhantomData,
        }
    }
}

impl<Val, Challenge, PackedChallenge, PCS, DFT, Chal> StarkConfig
    for StarkConfigImpl<Val, Challenge, PackedChallenge, PCS, DFT, Chal>
where
    Val: PrimeField64 + TwoAdicField,
    Challenge: ExtensionField<Val>,
    PackedChallenge: PackedField<Scalar = Challenge> + AbstractExtensionField<Val::Packing>,
    PCS: UnivariatePcs<Val, RowMajorMatrix<Val>, Chal>,
    DFT: TwoAdicSubgroupDft<Val>,
    Chal: FieldChallenger<Val> + Clone,
{
    type Val = Val;
    type Challenge = Challenge;
    type PackedChallenge = PackedChallenge;
    type PCS = PCS;
    type DFT = DFT;
    type Chal = Chal;

    fn pcs(&self) -> &Self::PCS {
        &self.pcs
    }

    fn dft(&self) -> &Self::DFT {
        &self.dft
    }

    fn challenger(&self) -> Self::Chal {
        self.init_challenger.clone()
    }
}
