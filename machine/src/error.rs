use core::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum VerificationError {
    /// The shape of openings does not match the chip shapes.
    InvalidProofShape(ProofShapeError),
    /// Opening proof is invalid.
    InvalidOpeningArgument,
    /// Out-of-domain evaluation mismatch.
    ///
    /// `constraints(zeta)` did not match `quotient(zeta) Z_H(zeta)`.
    OodEvaluationMismatch,
}

#[derive(Debug)]
pub struct OodEvaluationMismatch;

#[derive(Debug)]
pub enum ProofShapeError {
    Preprocessed,
    MainTrace,
    Permutation,
    Quotient,
}

impl Display for VerificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            VerificationError::InvalidProofShape(err) => {
                write!(f, "Invalid proof shape: for {} opening", err)
            }
            VerificationError::InvalidOpeningArgument => {
                write!(f, "Invalid opening argument")
            }
            VerificationError::OodEvaluationMismatch => {
                write!(f, "Out-of-domain evaluation mismatch")
            }
        }
    }
}

impl Display for ProofShapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ProofShapeError::Preprocessed => {
                write!(f, "Preprocessed opening mismatch")
            }
            ProofShapeError::MainTrace => {
                write!(f, "Main trace opening mismatch")
            }
            ProofShapeError::Permutation => {
                write!(f, "Permutation opening mismatch")
            }
            ProofShapeError::Quotient => {
                write!(f, "Quotient opening mismatch")
            }
        }
    }
}

impl From<ProofShapeError> for VerificationError {
    fn from(err: ProofShapeError) -> Self {
        VerificationError::InvalidProofShape(err)
    }
}

impl From<OodEvaluationMismatch> for VerificationError {
    fn from(_: OodEvaluationMismatch) -> Self {
        VerificationError::OodEvaluationMismatch
    }
}
