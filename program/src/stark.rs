// use crate::columns::{NUM_PREPROCESSED_COLS, NUM_PROGRAM_COLS};
// use crate::ProgramChip;
// use alloc::vec;
// use valida_lookups::LookupType;
// use valida_machine::InstructionWord;

// use p3_air::{Air, BaseAir, PairBuilder};
// use p3_field::Field;
// use p3_matrix::dense::RowMajorMatrix;

// impl<F, AB> Air<AB> for ProgramChip<F>
// where
//     AB: PairBuilder<F = F>,
//     F: Field,
// {
//     fn eval(&self, _builder: &mut AB) {}
// }

// impl<F: Field> BaseAir<F> for ProgramChip<F> {
//     fn width(&self) -> usize {
//         NUM_PROGRAM_COLS
//     }

//     fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
//         match self.lookup_type() {
//             LookupType::Preprocessed => Some(self.table()),
//             _ => None,
//         }
//     }
// }
