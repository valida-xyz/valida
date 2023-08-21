use core::marker::PhantomData;
use p3_challenger::FieldChallenger;
use p3_commit::UnivariatePcs;
use p3_dft::TwoAdicSubgroupDft;
use p3_field::{AbstractExtensionField, ExtensionField, Field, PackedField, TwoAdicField};
use p3_matrix::dense::RowMajorMatrix;

pub trait StarkConfig {
    /// The field over which trace data is encoded.
    type Val: Field;

    /// The domain over which trace polynomials are defined.
    type Domain: ExtensionField<Self::Val> + TwoAdicField;
    type PackedDomain: PackedField<Scalar = Self::Domain>;

    /// The field from which most random challenges are drawn.
    type Challenge: ExtensionField<Self::Val> + ExtensionField<Self::Domain> + TwoAdicField;
    type PackedChallenge: PackedField<Scalar = Self::Challenge>
        + AbstractExtensionField<Self::PackedDomain>;

    /// The PCS used to commit to trace polynomials.
    type PCS: for<'a> UnivariatePcs<
        Self::Val,
        Self::Domain,
        RowMajorMatrix<Self::Val>,
        Self::Challenger,
    >;

    type DFT: TwoAdicSubgroupDft<Self::Domain> + TwoAdicSubgroupDft<Self::Challenge>;

    /// The challenger (Fiat-Shamir) implementation used.
    type Challenger: FieldChallenger<Self::Val>;

    fn pcs(&self) -> &Self::PCS;

    fn dft(&self) -> &Self::DFT;

    fn challenger(&self) -> Self::Challenger;
}

pub struct StarkConfigImpl<Val, Domain, Challenge, PCS, DFT, Chal> {
    pcs: PCS,
    dft: DFT,
    init_challenger: Chal,
    _phantom_val: PhantomData<Val>,
    _phantom_domain: PhantomData<Domain>,
    _phantom_challenge: PhantomData<Challenge>,
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
            _phantom_domain: PhantomData,
            _phantom_challenge: PhantomData,
            _phantom_chal: PhantomData,
        }
    }
}

impl<Val, Domain, Challenge, PCS, DFT, Chal> StarkConfig
    for StarkConfigImpl<Val, Domain, Challenge, PCS, DFT, Chal>
where
    Val: Field,
    Domain: ExtensionField<Val> + TwoAdicField,
    Challenge: ExtensionField<Val> + ExtensionField<Domain> + TwoAdicField,
    Challenge::Packing: AbstractExtensionField<Domain::Packing>,
    PCS: UnivariatePcs<Val, Domain, RowMajorMatrix<Val>, Chal>,
    DFT: TwoAdicSubgroupDft<Domain> + TwoAdicSubgroupDft<Challenge>,
    Chal: FieldChallenger<Val> + Clone,
{
    type Val = Val;
    type Domain = Domain;
    type PackedDomain = Domain::Packing;
    type Challenge = Challenge;
    type PackedChallenge = Challenge::Packing;
    type PCS = PCS;
    type DFT = DFT;
    type Challenger = Chal;

    fn pcs(&self) -> &Self::PCS {
        &self.pcs
    }

    fn dft(&self) -> &Self::DFT {
        &self.dft
    }

    fn challenger(&self) -> Self::Challenger {
        self.init_challenger.clone()
    }
}
