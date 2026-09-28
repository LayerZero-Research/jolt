//! Wire round trips of the objects a deployed verifier receives.

use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    bincode::serde::encode_to_vec(value, bincode::config::standard())
        .expect("honest objects serialize")
}

/// Decode untrusted bytes; `None` for anything bincode rejects.
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    bincode::serde::decode_from_slice(bytes, bincode::config::standard())
        .ok()
        .map(|(value, _)| value)
}

/// Encode, decode, and require the decoded value to re-encode to the same
/// bytes: honest objects must survive transport unchanged.
pub fn roundtrip<T: Serialize + DeserializeOwned>(value: &T) -> T {
    let bytes = encode(value);
    let decoded: T = decode(&bytes).expect("honest objects deserialize");
    assert_eq!(encode(&decoded), bytes, "transport is not a fixed point");
    decoded
}
