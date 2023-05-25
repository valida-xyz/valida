use super::{Field, PrimeField, PrimeField64, MEMORY_CELL_BYTES};
use core::ops::{Add, Index, IndexMut, Mul, Sub};

#[derive(Copy, Clone, Debug, Default)]
pub struct Word<F>(pub [F; MEMORY_CELL_BYTES]);

impl Word<u8> {
    pub fn to_field<F: PrimeField>(&self) -> Word<F> {
        let mut word = Word::<F>::default();
        for i in 0..MEMORY_CELL_BYTES {
            word[i] = F::from_canonical_u8(self[i]);
        }
        word
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
