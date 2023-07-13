use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub enum Operation {
    PageIn(u32),
    PageOut(u32),
}

pub struct MemoryCache<N, F> {
    pub pages: BTreeMap<u32, MemoryPage<N, F>>,
}

pub struct MemoryPage<N, F> {
    pub data: [Word<F>; N],
}

impl<F> MemoryCache<F> {
    pub fn hash(&self) -> Hash {
        todo!()
    }
}

impl<F> MemoryPage<F> {
    pub fn new(data: Vec<Word<F>>) -> Self {
        Self { data }
    }
}
