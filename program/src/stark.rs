use crate::columns::NUM_PREPROCESSED_COLS;
use crate::ProgramChip;
use alloc::vec;
use valida_machine::InstructionWord;

use p3_air::{Air, BaseAir, PairBuilder};
use p3_field::PrimeField64;
use p3_matrix::dense::RowMajorMatrix;

impl<F, AB> Air<AB> for ProgramChip
where
    F: PrimeField64,
    AB: PairBuilder<F = F>,
{
    fn eval(&self, _builder: &mut AB) {}
}

impl<F: PrimeField64> BaseAir<F> for ProgramChip {
    fn preprocessed_trace(&self) -> Option<RowMajorMatrix<F>> {
        // Pad the ROM to a power of two.
        let mut rom = self.program_rom.0.clone();
        let n = rom.len();
        rom.resize(n.next_power_of_two(), InstructionWord::default());

        let flattened = rom
            .into_iter()
            .enumerate()
            .flat_map(|(n, word)| {
                let mut row = vec![F::ZERO; NUM_PREPROCESSED_COLS];
                row[0] = F::from_canonical_usize(n);
                row[1..].copy_from_slice(&word.flatten());
                row
            })
            .collect();
        let trace = RowMajorMatrix::new(flattened, NUM_PREPROCESSED_COLS);
        Some(trace)
    }
}
