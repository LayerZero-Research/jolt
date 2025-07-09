#![cfg_attr(feature = "guest", no_std)]

#[cfg(feature = "sha2_crate")]
use sha2::{Sha256, Digest};

#[jolt::provable]
fn sha2(input: &[u8]) -> [u8; 32] {
    #[cfg(feature = "jolt_sha256")]
    {
        assert_eq!(1, 2);
        // Use Jolt's optimized SHA256 implementation
        jolt::sha256::Sha256::digest(input)
    }
    
    #[cfg(feature = "sha2_crate")]
    {
        assert_eq!(1, 1);
        // Use standard sha2 crate
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.finalize().into()
    }
}
