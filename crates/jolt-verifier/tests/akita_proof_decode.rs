#![cfg(feature = "akita")]
#![expect(
    clippy::expect_used,
    reason = "test setup and decoding failures should identify their exact boundary"
)]

use std::sync::atomic::{AtomicUsize, Ordering};

use jolt_verifier::proof::{AkitaJointOpeningProof, MAX_AKITA_AUXILIARY_PROOFS};
use serde::{Deserialize, Deserializer, Serialize};

static DECODED_ELEMENTS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, PartialEq, Eq)]
struct CountedByte(u8);

impl<'de> Deserialize<'de> for CountedByte {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        let _ = DECODED_ELEMENTS.fetch_add(1, Ordering::SeqCst);
        Ok(Self(value))
    }
}

#[derive(Serialize)]
struct WireProof {
    main_batch: u8,
    auxiliary: Vec<u8>,
}

#[test]
fn auxiliary_count_is_bounded_before_auxiliary_elements_are_deserialized() {
    let config = bincode::config::standard();
    let at_limit = WireProof {
        main_batch: 7,
        auxiliary: vec![11; MAX_AKITA_AUXILIARY_PROOFS],
    };
    let encoded =
        bincode::serde::encode_to_vec(&at_limit, config).expect("at-limit proof should encode");

    DECODED_ELEMENTS.store(0, Ordering::SeqCst);
    let (decoded, consumed): (AkitaJointOpeningProof<CountedByte>, usize) =
        bincode::serde::decode_from_slice(&encoded, config)
            .expect("the maximum allowed auxiliary count should decode");
    assert_eq!(consumed, encoded.len());
    assert_eq!(decoded.main_batch, CountedByte(7));
    assert_eq!(decoded.auxiliary.len(), MAX_AKITA_AUXILIARY_PROOFS);
    assert_eq!(
        DECODED_ELEMENTS.load(Ordering::SeqCst),
        MAX_AKITA_AUXILIARY_PROOFS + 1
    );

    let oversized = WireProof {
        main_batch: 7,
        auxiliary: vec![11; MAX_AKITA_AUXILIARY_PROOFS + 1],
    };
    let encoded = bincode::serde::encode_to_vec(&oversized, config)
        .expect("oversized wire proof should encode");

    DECODED_ELEMENTS.store(0, Ordering::SeqCst);
    let error = bincode::serde::decode_from_slice::<AkitaJointOpeningProof<CountedByte>, _>(
        &encoded, config,
    )
    .expect_err("an oversized auxiliary count must be rejected");
    assert!(error
        .to_string()
        .contains("too many Akita auxiliary proofs"));
    assert_eq!(
        DECODED_ELEMENTS.load(Ordering::SeqCst),
        1,
        "the main batch may decode, but no auxiliary proof element may decode"
    );
}
