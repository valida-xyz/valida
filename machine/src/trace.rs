pub trait TraceGenerator<M, T> {
    const NUM_COLS: usize;

    /// Generate a trace for the chip given the provided machine.
    fn generate_trace(&self, machine: &M) -> Vec<[T; Self::NUM_COLS]>;
}
