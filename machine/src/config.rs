use p3_commit::UnivariatePCS;
use p3_field::ExtensionField;
use p3_field::PrimeField;

pub trait StarkConfig {
    /// The field over which trace data is encoded.
    type F: PrimeField;
    /// The field from which the verifier draws random challenges.
    type EF: ExtensionField<Self::F>;
    /// The polynomial commitment scheme used.
    type PCS: UnivariatePCS<Self::F>;
}
