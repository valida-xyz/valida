use super::{Field, PrimeField, MEMORY_CELL_BYTES};
use core::cmp::Ordering;
use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Index, IndexMut, Mul, Shl, Shr, Sub};

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
        let mut result = F::zero();
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

impl Div for Word<u8> {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        let b: u32 = self.into();
        let c: u32 = other.into();
        let res = b / c;
        res.into()
    }
}

impl Shl for Word<u8> {
    type Output = Self;
    fn shl(self, other: Self) -> Self {
        let b: u32 = self.into();
        let c: u32 = other.into();
        let res = b << c;
        res.into()
    }
}

impl Shr for Word<u8> {
    type Output = Self;
    fn shr(self, other: Self) -> Self {
        let b: u32 = self.into();
        let c: u32 = other.into();
        let res = b >> c;
        res.into()
    }
}

impl BitXor for Word<u8> {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        let mut res = self;
        for i in 0..MEMORY_CELL_BYTES {
            res[i] ^= other[i];
        }
        res
    }
}

impl BitAnd for Word<u8> {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        let mut res = self;
        for i in 0..MEMORY_CELL_BYTES {
            res[i] &= other[i];
        }
        res
    }
}

impl BitOr for Word<u8> {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        let mut res = self;
        for i in 0..MEMORY_CELL_BYTES {
            res[i] |= other[i];
        }
        res
    }
}

impl<F: Field> From<F> for Word<F> {
    fn from(bytes: F) -> Self {
        Self([F::zero(), F::zero(), F::zero(), bytes])
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

impl<F: Ord> PartialOrd for Word<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: Ord> Ord for Word<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| a.cmp(b))
            .find(|&ord| ord != Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    }
}

impl<F> IntoIterator for Word<F> {
    type Item = F;
    type IntoIter = core::array::IntoIter<F, MEMORY_CELL_BYTES>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
