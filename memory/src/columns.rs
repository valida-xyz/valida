use valida_machine::Word;

pub struct MemoryCols<T> {
    pub clk: T,
    pub addr: T,
    pub value: Word<T>,
    pub clk_lo: T, // Lower 16 bits of clk' - clk (sorted)
    pub clk_hi: T, // Upper 16 bits of clk' - clk (sorted)
    pub t: T,      // Nondeterministic inverse of addr' - addr (sorted)
}

pub struct MemoryLog<T> {
    clk: T,
    addr: T,
    value: Word<T>,
    is_write: bool,
}

impl<T> MemoryLog<T> {
    pub fn new<A: Into<T>, V: Into<Word<T>>>(clk: T, addr: A, value: V, is_write: bool) -> Self {
        Self {
            clk,
            addr: addr.into(),
            value: value.into(),
            is_write,
        }
    }
}
