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
    type Pcs: for<'a> UnivariatePcs<
        Self::Val,
        Self::Domain,
        Self::Challenge,
        RowMajorMatrix<Self::Val>,
        Self::Challenger,
    >;

    type Dft: TwoAdicSubgroupDft<Self::Domain> + TwoAdicSubgroupDft<Self::Challenge>;

    /// The challenger (Fiat-Shamir) implementation used.
    type Challenger: FieldChallenger<Self::Val>;

    fn pcs(&self) -> &Self::Pcs;

    fn dft(&self) -> &Self::Dft;

    fn challenger(&self) -> Self::Challenger;
}

pub struct StarkConfigImpl<Val, Domain, Challenge, Pcs, Dft, Chal> {
    pcs: Pcs,
    dft: Dft,
    init_challenger: Chal,
    _phantom_val: PhantomData<Val>,
    _phantom_domain: PhantomData<Domain>,
    _phantom_challenge: PhantomData<Challenge>,
    _phantom_chal: PhantomData<Chal>,
}

impl<Val, Challenge, PackedChallenge, Pcs, Dft, Chal>
    StarkConfigImpl<Val, Challenge, PackedChallenge, Pcs, Dft, Chal>
{
    pub fn new(pcs: Pcs, dft: Dft, init_challenger: Chal) -> Self {
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

impl<Val, Domain, Challenge, Pcs, Dft, Challenger> StarkConfig
    for StarkConfigImpl<Val, Domain, Challenge, Pcs, Dft, Challenger>
where
    Val: Field,
    Domain: ExtensionField<Val> + TwoAdicField,
    Challenge: ExtensionField<Val> + ExtensionField<Domain> + TwoAdicField,
    Challenge::Packing: AbstractExtensionField<Domain::Packing>,
    Pcs: UnivariatePcs<Val, Domain, Challenge, RowMajorMatrix<Val>, Challenger>,
    Dft: TwoAdicSubgroupDft<Domain> + TwoAdicSubgroupDft<Challenge>,
    Challenger: FieldChallenger<Val> + Clone,
{
    type Val = Val;
    type Domain = Domain;
    type PackedDomain = Domain::Packing;
    type Challenge = Challenge;
    type PackedChallenge = Challenge::Packing;
    type Pcs = Pcs;
    type Dft = Dft;
    type Challenger = Challenger;

    fn pcs(&self) -> &Self::Pcs {
        &self.pcs
    }

    fn dft(&self) -> &Self::Dft {
        &self.dft
    }

    fn challenger(&self) -> Self::Challenger {
        self.init_challenger.clone()
    }
}
