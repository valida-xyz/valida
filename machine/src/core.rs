use super::{Field, PrimeField, PrimeField32, MEMORY_CELL_BYTES};
use core::ops::{Add, Index, IndexMut, Mul, Sub};

#[derive(Copy, Clone, Debug, Default)]
pub struct Word<F>(pub [F; MEMORY_CELL_BYTES]);

impl<F: PrimeField> Word<F> {
    pub fn to_value(&self) -> F {
        let mut value = F::ZERO;
        for i in 0..MEMORY_CELL_BYTES {
            value = self.0[i] + value * F::from_canonical_u32(256);
        }
        value
    }
}

impl<F: PrimeField32> Add for Word<F> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let mut a = Word::<F>::default();
        let mut carry = 0u8;
        for i in (0..MEMORY_CELL_BYTES).rev() {
            let b_i = self[i].as_canonical_u32() as u8;
            let c_i = other[i].as_canonical_u32() as u8;
            let (sum, overflow) = b_i.overflowing_add(c_i);
            let (sum_with_carry, carry_overflow) = sum.overflowing_add(carry);
            carry = overflow as u8 + carry_overflow as u8;
            a[i] = F::from_canonical_u8(sum_with_carry);
        }
        a
    }
}

impl<F: PrimeField> Sub for Word<F> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        todo!()
    }
}

impl<F: PrimeField32> Mul for Word<F> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut a = Word::<F>::default();
        let b: [u32; 4] = self.into();
        let c: [u32; 4] = rhs.into();
        let res = b[3] * c[3]
            + ((b[3] * c[2] + b[2] * c[3]) << 8)
            + ((b[3] * c[1] + b[2] * c[2] + b[1] * c[3]) << 16)
            + ((b[3] * c[0] + b[2] * c[1] + b[1] * c[2] + b[0] * c[3]) << 24);
        a[0] = F::from_canonical_u32(res & 0xff);
        a[1] = F::from_canonical_u32((res >> 8) & 0xff);
        a[2] = F::from_canonical_u32((res >> 16) & 0xff);
        a[3] = F::from_canonical_u32((res >> 24) & 0xff);
        a
    }
}

impl<F: PrimeField> From<[u32; MEMORY_CELL_BYTES]> for Word<F> {
    fn from(bytes: [u32; MEMORY_CELL_BYTES]) -> Word<F> {
        let mut result = Word::default();
        for i in 0..MEMORY_CELL_BYTES {
            result[i] = F::from_canonical_u32(bytes[i])
        }
        result
    }
}

impl<F: PrimeField32> Into<[u32; MEMORY_CELL_BYTES]> for Word<F> {
    fn into(self) -> [u32; MEMORY_CELL_BYTES] {
        let mut result = [0; MEMORY_CELL_BYTES];
        for i in 0..MEMORY_CELL_BYTES {
            result[i] = self.0[i].as_canonical_u32();
        }
        result
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

impl<F> Eq for Word<F> where F: Field {}

impl<F> PartialEq for Word<F>
where
    F: Field,
{
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
