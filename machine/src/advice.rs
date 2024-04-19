use core::slice;
use std::io;
use std::io::Read;
use std::fs::File;

pub trait AdviceProvider {
    /// Get the next byte from the advice tape, if any.
    fn get_advice(&mut self) -> Option<u8>;
}

pub struct FixedAdviceProvider {
    advice: Vec<u8>,
    index: usize,
}

enum AdviceProviderType {
    Stdin(StdinAdviceProvider),
    Fixed(FixedAdviceProvider),
}

impl AdviceProviderType {
    fn get_advice(&mut self) -> Option<u8> {
        match self {
            AdviceProviderType::Stdin(provider) => provider.get_advice(),
            AdviceProviderType::Fixed(provider) => provider.get_advice(),
        }
    }
}

impl FixedAdviceProvider {
    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn new(advice: Vec<u8>) -> Self {
        Self { advice, index: 0 }
    }
    pub fn from_file(file : &mut File) -> Self {
        // read the entire file into self::advice:
        let mut advice = Vec::new();
        file.read_to_end(&mut advice).unwrap();
        Self { advice, index: 0 }
    }
}

impl AdviceProvider for FixedAdviceProvider {
    fn get_advice(&mut self) -> Option<u8> {
        if self.index < self.advice.len() {
            let advice_byte = self.advice[self.index];
            self.index += 1;
            Some(advice_byte)
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
pub struct StdinAdviceProvider;

#[cfg(feature = "std")]
impl AdviceProvider for StdinAdviceProvider {
    fn get_advice(&mut self) -> Option<u8> {
        let mut advice_byte = 0u8;
        match io::stdin().read_exact(slice::from_mut(&mut advice_byte)) {
            Ok(_) => Some(advice_byte),
            Err(_) => None,
        }
    }
}
