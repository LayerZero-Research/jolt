//! Infallible, mutation-local byte decoding.
//!
//! Every read succeeds: an exhausted input yields zeros. Harnesses therefore
//! never reject a case for being short, and a byte change affects only the
//! field it encodes. Fixed-width tokens keep later fields at stable offsets
//! when earlier values change.

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn fill(&mut self, out: &mut [u8]) {
        let available = self.remaining().min(out.len());
        if available > 0 {
            out[..available].copy_from_slice(&self.data[self.pos..self.pos + available]);
        }
        out[available..].fill(0);
        self.pos = self.pos.saturating_add(out.len());
    }

    pub fn bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut out = [0u8; N];
        self.fill(&mut out);
        out
    }

    pub fn take(&mut self, len: usize) -> &'a [u8] {
        let start = self.pos.min(self.data.len());
        let end = self.pos.saturating_add(len).min(self.data.len());
        self.pos = self.pos.saturating_add(len);
        &self.data[start..end]
    }

    pub fn u8(&mut self) -> u8 {
        self.bytes::<1>()[0]
    }

    pub fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.bytes())
    }

    pub fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.bytes())
    }

    pub fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.bytes())
    }
}

/// Deterministic expansion for bulk entries not given explicitly.
#[derive(Clone)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn next_u128(&mut self) -> u128 {
        (u128::from(self.next_u64()) << 64) | u128::from(self.next_u64())
    }
}
