//! Preprocessing-time schedule planning (liveness).
//!
//! Input: a production preprocessing [`Shape`]. Every shape inside Jolt's
//! documented limits must plan its grouped Akita rows; when the opening is
//! small enough for one fuzz iteration it must also commit, prove, and verify
//! with honest data. A failure of an in-contract shape at any stage, clean
//! error or not, is a finding. Out-of-contract shapes are only counted.

use crate::input::Reader;
use crate::opening::{self, Witness};
use crate::shape::{self, Shape};
use crate::{artifacts, env, stats};

pub fn run(data: &[u8]) {
    env::init();
    let mut reader = Reader::new(data);
    let shape = Shape::decode(&mut reader);
    let witness = Witness::decode(&mut reader);
    check(&shape, &witness);
}

pub fn check(shape: &Shape, witness: &Witness) {
    let artifacts = artifacts::shared();
    let in_contract = shape.in_contract();
    let result = shape.setup_request().and_then(|request| {
        stats::time("plan", || shape::plan(&request))?;
        stats::count("planned");
        if request.total_coefficients() <= env::max_case_coefficients() {
            env::on_large_stack(|| {
                opening::commit(&artifacts, &request, witness)?.prove_and_verify(&shape.to_string())
            })?;
        } else {
            stats::count("prove_skipped_cost");
        }
        Ok(())
    });
    match result {
        Ok(()) if in_contract => stats::count("in_contract_ok"),
        Ok(()) => stats::count("out_of_contract_accepted"),
        Err(failure) if in_contract => {
            panic!("liveness: in-contract preprocessing shape [{shape}] failed at {failure}")
        }
        Err(_) => stats::count("out_of_contract_rejected"),
    }
}

/// The smallest production shape of every structural case: no advice, each
/// advice kind at the default 4096 bytes, both, and a committed program with
/// one and two chunks (the fixture layouts); both advice kinds under every
/// multi-chunk profile; and the K=256 override at a listed trace group.
pub fn seeds() -> Vec<(String, Vec<u8>)> {
    use crate::opening::Fill;
    use crate::shape::{Chunking, CommittedProgram, MIN_LOG_T};
    use jolt_akita::AkitaChunkProfile;
    let witness = Witness {
        seed: 7,
        dense: Fill::Random,
        trace: Fill::Random,
        zero_committed_columns: 0,
        point: Fill::Random,
    };
    let base = Shape {
        log_t: MIN_LOG_T,
        chunking: Chunking::Production,
        profile: AkitaChunkProfile::Single,
        log_bytecode_len: 10,
        log_ram_k: 20,
        untrusted_advice_bytes: None,
        trusted_advice_bytes: None,
        program: None,
    };
    let mut shapes = Vec::new();
    for (case, untrusted, trusted, program) in [
        ("plain", None, None, None),
        ("untrusted", Some(4096), None, None),
        ("trusted", None, Some(4096), None),
        ("both", Some(4096), Some(4096), None),
        ("committed1", None, None, Some(0)),
        ("committed2-both", Some(4096), Some(4096), Some(1)),
    ] {
        shapes.push((
            format!("k16-{case}"),
            Shape {
                untrusted_advice_bytes: untrusted,
                trusted_advice_bytes: trusted,
                program: program.map(|log_chunks| CommittedProgram {
                    log_chunks,
                    image_words: 300,
                }),
                ..base
            },
        ));
    }
    for profile in [
        AkitaChunkProfile::Two,
        AkitaChunkProfile::Four,
        AkitaChunkProfile::Eight,
    ] {
        shapes.push((
            format!("k16-{profile:?}-both").to_lowercase(),
            Shape {
                profile,
                untrusted_advice_bytes: Some(4096),
                trusted_advice_bytes: Some(4096),
                ..base
            },
        ));
    }
    // log_T 12 at K=256 with two bytecode and two RAM chunks: the listed
    // 20-variable, 29-column trace group.
    shapes.push((
        "k256-plain".to_string(),
        Shape {
            chunking: Chunking::ForcedK256,
            log_ram_k: 16,
            ..base
        },
    ));
    shapes
        .into_iter()
        .map(|(name, shape)| {
            let mut bytes = shape.encode();
            bytes.extend(witness.encode());
            (name, bytes)
        })
        .collect()
}
