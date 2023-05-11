pub trait TraceGenerator<M> {
    type F;
    type FE;
    const NUM_COLS: usize;
    const NUM_PERM_COLS: usize;

    /// Generate the main trace for the chip given the provided machine.
    fn generate_trace(&self, machine: &M) -> Vec<[Self::F; Self::NUM_COLS]>;

    /// Generate the permutation trace for the chip given the provided machine.
    fn generate_permutation_trace(
        &self,
        machine: &M,
        main_trace: Vec<[Self::F; Self::NUM_COLS]>,
        random_elements: Vec<Self::FE>,
    ) -> Vec<[Self::FE; Self::NUM_PERM_COLS]>;
}
