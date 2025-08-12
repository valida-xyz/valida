use super::MEMORY_CELL_BYTES;
use core::cmp::Ordering;
use core::mem::transmute;
use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Index, IndexMut, Mul, Shl, Shr, Sub};
use p3_field::{Field, PrimeField};

// Currently stored in big-endian form.
#[derive(Copy, Clone, Debug, Default)]
pub struct Word<F>(pub [F; MEMORY_CELL_BYTES]);

// Functions for byte manipulations
/// Get the index of a byte in a memory cell.
/// The index is converted from little endian (as emitted by the compiler) to big endian.
pub fn index_of_byte(addr: u32) -> usize {
    (3 - (addr & 3)) as usize
}

/// Get the address of the memory cells which is not empty (a multiple of 4).
pub fn addr_of_word(addr: u32) -> u32 {
    (addr & !3) as u32
}

pub fn is_mul_4(addr: u32) -> bool {
    addr.rem_euclid(4) == 0
}
//----------------------------------

impl Word<u8> {
    pub fn from_u8(byte: u8) -> Self {
        let mut result = [0; MEMORY_CELL_BYTES];
        result[MEMORY_CELL_BYTES - 1] = byte;
        Self(result)
    }
}

impl Word<u8> {
    pub fn sign_extend_byte(byte: u8) -> Self {
        let sign = byte as i8 >> 7;
        let mut result: [u8; MEMORY_CELL_BYTES] = [sign as u8; MEMORY_CELL_BYTES];
        result[3] = byte;
        Self(result)
    }
}

// The cell is stored in little endian format in the compiler. But the VM stores it in big endian.
impl Word<u8> {
    pub fn update_byte(self, byte: u8, loc: usize) -> Self {
        let result_little_end: [u8; MEMORY_CELL_BYTES] = self.0;
        let mut result = [0; 4];
        // Convert from little to big endian.
        for i in 0..MEMORY_CELL_BYTES {
            result[i] = result_little_end[3 - i];
        }
        result[loc] = byte;
        Self(result)
    }
}

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

impl Into<i32> for Word<u8> {
    fn into(self) -> i32 {
        unsafe { transmute::<u32, i32>(self.into()) }
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
        let res=b.wrapping_add(c);
        res.into()
    }
}

impl Sub for Word<u8> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let b: u32 = self.into();
        let c: u32 = other.into();
        let res = b.wrapping_sub(c);
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

pub trait Mulhs<Rhs = Self> {
    /// The resulting type after applying the `/` operator.
    type Output;

    fn mulhs(self, rhs: Rhs) -> Self::Output;
}

impl Mulhs for Word<u8> {
    type Output = Self;
    fn mulhs(self, other: Self) -> Self {
        let bu32: u32 = self.into();
        let bi64 = bu32 as i64;
        let cu32: u32 = other.into();
        let ci64 = cu32 as i64;
        // The result of regular multiplication represented in i64
        let mul_res = bi64 * ci64;
        let res = (mul_res >> 32) as i32 as u32;
        res.into()
    }
}

pub trait Mulhu<Rhs = Self> {
    /// The resulting type after applying the `/` operator.
    type Output;

    fn mulhu(self, rhs: Rhs) -> Self::Output;
}

impl Mulhu for Word<u8> {
    type Output = Self;
    fn mulhu(self, other: Self) -> Self {
        let bu32: u32 = self.into();
        let bu64 = bu32 as u64;
        let cu32: u32 = other.into();
        let cu64 = cu32 as u64;
        // The result of regular multiplication represented in u64
        let mul_res = bu64 * cu64;
        let res = (mul_res >> 32) as u32;
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

pub trait SDiv<Rhs = Self> {
    /// The resulting type after applying the `/` operator.
    type Output;

    fn sdiv(self, rhs: Rhs) -> Self::Output;
}

impl SDiv for Word<u8> {
    type Output = Self;
    fn sdiv(self, other: Self) -> Self {
        let bu: u32 = self.into();
        let b = bu as i32;
        let cu: u32 = other.into();
        let c = cu as i32;
        // perform the division in i32 first, then convert it to u32
        let res = (b / c) as u32;
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

pub trait Sra<Rhs = Self> {
    /// The resulting type after applying the `/` operator.
    type Output;

    fn sra(self, rhs: Rhs) -> Self::Output;
}

impl Sra for Word<u8> {
    type Output = Self;
    fn sra(self, other: Self) -> Self {
        let bu: u32 = self.into();
        let b = bu as i32;
        let cu: u32 = other.into();
        let c = cu as i32;
        // See https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators
        // >> Performs arithmetic right shift on signed integer types, logical right shift on unsigned integer types.
        // TODO: This may panic on overflow. LLVM says `ashr` overflow should result in a poison value.
        // I think it's fine to return something like `b >> (c % 0b11111)`, but not to immediately panic.
        let res = (b >> c) as u32;
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
