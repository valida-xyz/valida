use super::{Field, PrimeField, PrimeField64, MEMORY_CELL_BYTES};
use core::ops::{Add, Index, IndexMut, Mul, Sub};

#[derive(Copy, Clone, Debug, Default)]
pub struct Word<F>(pub [F; MEMORY_CELL_BYTES]);

impl<F: Copy> Word<F> {
    pub fn transform<T, G>(self, mut f: G) -> Word<T>
    where
        G: FnMut(F) -> T,
        T: Default + Copy,
    {
        let mut result: [T; MEMORY_CELL_BYTES] = [T::default(); MEMORY_CELL_BYTES];
        for (i, item) in self.0.iter().enumerate() {
            result[i] = f(*item);
        }
        Word(result)
    }
}

impl<F: PrimeField> Word<F> {
    pub fn reduce(self) -> F {
        let mut result = F::ZERO;
        for (n, item) in self.0.into_iter().rev().enumerate() {
            result = result + item * F::from_canonical_u32(1 << 8 * n);
        }
        result
    }
}

impl Into<u32> for Word<u8> {
    fn into(self) -> u32 {
        let mut result = 0u32;
        for i in 0..MEMORY_CELL_BYTES {
            result += (self[MEMORY_CELL_BYTES - i - 1] as u32) << (i * 8);
        }
        result
    }
}

impl From<u32> for Word<u8> {
    fn from(value: u32) -> Self {
        let mut result = Word::<u8>::default();
        for i in 0..MEMORY_CELL_BYTES {
            result[MEMORY_CELL_BYTES - i - 1] = ((value >> (i * 8)) & 0xFF) as u8;
        }
        result
    }
}

impl Add for Word<u8> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let b: u32 = self.into();
        let c: u32 = other.into();
        let res = (b as u64 + c as u64) as u32;
        res.into()
    }
}

impl Sub for Word<u8> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let b: u32 = self.into();
        let c: u32 = other.into();
        let res = b - c;
        res.into()
    }
}

impl Mul for Word<u8> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let b: u32 = self.into();
        let c: u32 = other.into();
        let res = b * c;
        res.into()
    }
}

impl<F: Field> From<F> for Word<F> {
    fn from(bytes: F) -> Self {
        Self([F::ZERO, F::ZERO, F::ZERO, bytes])
    }
}

impl<T> Index<usize> for Word<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for Word<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<F: Ord> Eq for Word<F> {}

impl<F: Ord> PartialEq for Word<F> {
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
    }
}

impl<F> IntoIterator for Word<F> {
    type Item = F;
    type IntoIter = core::array::IntoIter<F, MEMORY_CELL_BYTES>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
