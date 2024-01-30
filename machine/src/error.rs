use core::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum VerificationError {
    /// The shape of opennings does not match the chip shapes.
    InvalidProofShape(ProofShapeError),
    /// Openning proof is invalid.
    InvalidOpenningArgument,
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
    Permuation,
    Quotient,
}

impl Display for VerificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            VerificationError::InvalidProofShape(err) => {
                write!(f, "Invalid proof shape: for {} openning", err)
            }
            VerificationError::InvalidOpenningArgument => {
                write!(f, "Invalid openning argument")
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
            ProofShapeError::Permuation => {
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
