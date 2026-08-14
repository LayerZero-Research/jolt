//! Canonical trusted/untrusted advice word encoding.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::jolt_device::bytes_to_words_le;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdviceWordsError {
    CapacityNotWordAligned { max_bytes: usize },
    CapacityNotPowerOfTwo { max_bytes: usize },
    AdviceTooLong { actual: usize, max: usize },
}

impl fmt::Display for AdviceWordsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityNotWordAligned { max_bytes } => write!(
                formatter,
                "advice capacity {max_bytes} is not aligned to an 8-byte word"
            ),
            Self::CapacityNotPowerOfTwo { max_bytes } => write!(
                formatter,
                "nonzero advice capacity {max_bytes} is not a power of two"
            ),
            Self::AdviceTooLong { actual, max } => {
                write!(
                    formatter,
                    "advice has {actual} bytes, exceeding configured max {max}"
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AdviceWordsError {}

/// Number of little-endian `u64` words in the canonical advice domain.
///
/// A zero byte capacity retains the protocol's one-coefficient empty domain;
/// nonzero capacities must already be the aligned power-of-two value carried
/// by `MemoryLayout`.
pub fn advice_word_capacity(max_bytes: usize) -> Result<usize, AdviceWordsError> {
    if !max_bytes.is_multiple_of(8) {
        return Err(AdviceWordsError::CapacityNotWordAligned { max_bytes });
    }
    if max_bytes != 0 && !max_bytes.is_power_of_two() {
        return Err(AdviceWordsError::CapacityNotPowerOfTwo { max_bytes });
    }
    Ok((max_bytes / 8).max(1))
}

/// Packs bytes into the canonical zero-padded little-endian advice word table.
pub fn canonical_advice_words(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<Vec<u64>, AdviceWordsError> {
    let word_capacity = advice_word_capacity(max_bytes)?;
    if bytes.len() > max_bytes {
        return Err(AdviceWordsError::AdviceTooLong {
            actual: bytes.len(),
            max: max_bytes,
        });
    }
    let mut words = bytes_to_words_le(bytes);
    words.resize(word_capacity, 0);
    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_words_are_little_endian_and_zero_padded() {
        assert_eq!(canonical_advice_words(&[], 0), Ok(vec![0]));
        assert_eq!(
            canonical_advice_words(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 16),
            Ok(vec![0x0807_0605_0403_0201, 9])
        );
        assert_eq!(
            canonical_advice_words(&[u8::MAX; 8], 16),
            Ok(vec![u64::MAX, 0])
        );
    }

    #[test]
    fn invalid_capacity_and_oversize_are_rejected() {
        assert_eq!(
            canonical_advice_words(&[], 7),
            Err(AdviceWordsError::CapacityNotWordAligned { max_bytes: 7 })
        );
        assert_eq!(
            canonical_advice_words(&[], 24),
            Err(AdviceWordsError::CapacityNotPowerOfTwo { max_bytes: 24 })
        );
        assert_eq!(
            canonical_advice_words(&[0; 9], 8),
            Err(AdviceWordsError::AdviceTooLong { actual: 9, max: 8 })
        );
    }
}
