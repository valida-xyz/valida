//! Items intended to be used only by `valida-derive`.

pub type DefaultField = BabyBear;
pub type DefaultExtensionField = BabyBear; // FIXME: Replace

mod check_constraints;
mod debug_builder;
mod folding_builder;
mod prove;

pub use check_constraints::*;
pub use debug_builder::*;
pub use folding_builder::*;
use p3_baby_bear::BabyBear;
pub use prove::*;

// Re-export some Plonky3 crates so that derives can use them.
pub use p3_challenger;
pub use p3_commit;
pub use p3_matrix;
pub use p3_util;
