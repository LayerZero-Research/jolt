//! A transported verifier setup carries its exact schedule catalog as bytes. A
//! catalog with an out-of-range digit base must be rejected when the verifier
//! admits it, not abort the verifier process.

#![expect(clippy::expect_used, reason = "tests assert successful proof setup")]

#[expect(
    dead_code,
    reason = "shared integration-test support is compiled independently per test file"
)]
mod support;

use jolt_akita::{AkitaScheme, AkitaVerifierSetup};
use jolt_openings::CommitmentScheme;
use jolt_transcript::{Blake2bTranscript, Transcript};
use serde_json::Value;
use support::{f, layout, polynomial, setup_for};

/// The verifier setup's JSON with the dense catalog's first terminal digit
/// base replaced by `log_basis`.
fn setup_with_terminal_log_basis(setup: &AkitaVerifierSetup, log_basis: u32) -> AkitaVerifierSetup {
    let mut value = serde_json::to_value(setup).expect("verifier setup JSON");
    let dense = &mut value["schedule_artifacts"]["both"]["dense"];
    let bytes: Vec<u8> = serde_json::from_value(dense.clone()).expect("dense catalog bytes");
    let mut catalog: Value = serde_json::from_slice(&bytes).expect("catalog JSON");
    catalog["rows"][0]["schedule"]["terminal"]["inner"]["digits"]["log_basis"] = log_basis.into();
    *dense = serde_json::to_value(serde_json::to_vec_pretty(&catalog).expect("catalog JSON"))
        .expect("dense catalog bytes");
    serde_json::from_value(value).expect("tampered verifier setup deserializes")
}

#[test]
fn transported_setup_with_out_of_range_catalog_log_basis_is_rejected() {
    let (prover_setup, verifier_setup) = setup_for(14, 1, layout(7));
    let poly = polynomial(14, 100);
    let point: Vec<_> = (0..14).map(|i| f(2 + 3 * i)).collect();
    let eval = poly.evaluate(&point);
    let (commitment, hint) = AkitaScheme::commit(&poly, &prover_setup).expect("commit");
    let proof = AkitaScheme::open(
        &poly,
        &point,
        eval,
        &prover_setup,
        Some(hint),
        &mut Blake2bTranscript::new(b"akita-setup-admission"),
    )
    .expect("honest proof");

    for log_basis in [0, 128, u32::MAX] {
        let tampered = setup_with_terminal_log_basis(&verifier_setup, log_basis);
        let result = AkitaScheme::verify(
            &commitment,
            &point,
            eval,
            &proof,
            &tampered,
            &mut Blake2bTranscript::new(b"akita-setup-admission"),
        );
        assert!(
            result.is_err(),
            "a catalog with terminal log_basis {log_basis} must be rejected"
        );
    }
}
