use p3_commit::pcs::PCS;
use p3_field::field::Field;

pub struct MultistarkProof<F: Field, P: PCS<F>> {
    opening_proof: P::Proof,
}
