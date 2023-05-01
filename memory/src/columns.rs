use valida_machine::Word;

pub struct MemoryCols<T> {
    pub clk: T,
    pub addr: T,
    pub value: Word<T>,
}
