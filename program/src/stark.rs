use crate::ProgramChip;

use p3_air::{Air, AirBuilder};
use p3_matrix::dense::RowMajorMatrix;
use valida_machine::{InstructionWord, INSTRUCTION_ELEMENTS};

impl<AB> Air<AB> for ProgramChip<AB::F>
where
    AB: AirBuilder,
{
    fn eval(&self, _builder: &mut AB) {
        // TODO
    }

    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<AB::F>> {
        // Pad the ROM to a power of two.
        let mut rom = self.program_rom.0.clone();
        let n = rom.len();
        rom.resize(n.next_power_of_two(), InstructionWord::default());

        let flattened = rom.into_iter().flat_map(|word| word.flatten()).collect();
        let trace = RowMajorMatrix::new(flattened, INSTRUCTION_ELEMENTS);
        Some(trace)
    }
}
