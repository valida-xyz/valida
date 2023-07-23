use valida_machine::Operands;

#[derive(Default)]
pub struct ProgramCols<T> {
    pub multiplicity: T,
}

pub struct ProgramPreprocessedCols<T> {
    pub opcode: T,
    pub operands: Operands<T>,
}
