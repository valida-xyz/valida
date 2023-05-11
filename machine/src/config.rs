use p3_commit::UnivariatePCS;
use p3_field::{Field, PrimeField};

pub trait StarkConfig {
    /// The field over which trace data is encoded.
    type F: PrimeField;
    /// The field from which the verifier draws random challenges.
    type FE: Field<Base = Self::F>;
    /// The polynomial commitment scheme used.
    type PCS: UnivariatePCS<Self::F>;
}
