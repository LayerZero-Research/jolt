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
        stats::time("plan", || shape::plan(&artifacts, &request))?;
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
/// one and two chunks (the fixture layouts), at K=16 and forced K=256.
pub fn seeds() -> Vec<(String, Vec<u8>)> {
    use crate::opening::Fill;
    use crate::shape::{Chunking, CommittedProgram};
    let witness = Witness {
        seed: 7,
        dense: Fill::Random,
        trace: Fill::Random,
        columns: 63,
        zero_committed_columns: 0,
        point: Fill::Random,
    };
    let mut seeds = Vec::new();
    for (chunk_name, chunking) in [
        ("k16", Chunking::Production),
        ("k256", Chunking::Forced { log_k_chunk: 8 }),
    ] {
        for (case, untrusted, trusted, program) in [
            ("plain", None, None, None),
            ("untrusted", Some(4096), None, None),
            ("trusted", None, Some(4096), None),
            ("both", Some(4096), Some(4096), None),
            ("committed1", None, None, Some(0)),
            ("committed2-both", Some(4096), Some(4096), Some(1)),
        ] {
            let shape = Shape {
                log_t: crate::shape::MIN_LOG_T,
                chunking,
                log_bytecode_len: 10,
                log_ram_k: 20,
                untrusted_advice_bytes: untrusted,
                trusted_advice_bytes: trusted,
                program: program.map(|log_chunks| CommittedProgram {
                    log_chunks,
                    image_words: 300,
                }),
            };
            let mut bytes = shape.encode();
            bytes.extend(witness.encode());
            seeds.push((format!("{chunk_name}-{case}"), bytes));
        }
    }
    seeds
}
