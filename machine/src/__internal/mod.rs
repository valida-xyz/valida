use p3_mersenne_31::Mersenne31;

pub type DefaultField = Mersenne31;
pub type DefaultExtensionField = Mersenne31; // FIXME: Replace

mod check_constraints;
mod debug_builder;

pub use check_constraints::*;
pub use debug_builder::*;
