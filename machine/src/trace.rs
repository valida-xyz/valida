pub trait TraceGenerator<M, T> {
    const NUM_COLS: usize;
    type Operation;

    /// Generate a trace for the chip given the provided machine.
    fn generate_trace(&self, machine: &M) -> Vec<[T; Self::NUM_COLS]>;

    /// Convert an operation to a trace row.
    fn op_to_row<N: Into<usize>>(
        &self,
        n: N,
        op: Self::Operation,
        machine: &M,
    ) -> [T; Self::NUM_COLS];
}
