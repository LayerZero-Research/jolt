//! The Jolt verifier on untrusted bytes (robustness, then soundness).
//!
//! A bundle is what a deployed verifier receives: the verifier preprocessing
//! (including the Akita verifier setup with its exact schedule catalog), the
//! public I/O, the proof, and the trusted-advice commitment, each bincode
//! encoded. Honest bundles are proved once by `jolt-fuzz-dev bundles` and
//! shipped with the campaign.
//!
//! The input picks a bundle and a region and edits that region's bytes; the
//! other regions stay honest, so every edit reaches deserialization or
//! verification of exactly one object. The oracle:
//!
//! - decoding and verification never panic (libFuzzer's malloc limit also
//!   bounds what a decoded setup may allocate while re-deriving keys);
//! - a bundle that verifies must decode to an honest bundle, up to the
//!   representation choices the verifier does not bind (`Bundle::semantic`):
//!   accepting any other statement or proof is a soundness finding. Accepted
//!   alternative encodings of an honest bundle are counted
//!   (`noncanonical_encoding_accepted`, `equivalent_statement_accepted`).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use common::jolt_device::JoltDevice;
use jolt_akita::{AkitaCommitment, AkitaField, AkitaScheme};
use jolt_prover::akita::preprocessing::{AkitaTranscript, AkitaVc, AkitaVerifierPreprocessing};
use jolt_verifier::proof::JoltProof;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::input::Reader;
use crate::{env, stats, transport};

pub const BUNDLES_ENV: &str = "JOLT_FUZZ_BUNDLES";

type Proof = JoltProof<AkitaScheme, AkitaVc>;

/// The four objects a verifier receives, each as its own byte string.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub name: String,
    pub preprocessing: Vec<u8>,
    pub public_io: Vec<u8>,
    pub proof: Vec<u8>,
    pub trusted_advice_commitment: Option<Vec<u8>>,
}

impl Bundle {
    pub fn new(
        name: &str,
        preprocessing: &AkitaVerifierPreprocessing,
        public_io: &JoltDevice,
        proof: &Proof,
        trusted_advice_commitment: Option<&AkitaCommitment>,
    ) -> Self {
        Self {
            name: name.to_string(),
            preprocessing: transport::encode(preprocessing),
            public_io: transport::encode(public_io),
            proof: transport::encode(proof),
            trusted_advice_commitment: trusted_advice_commitment.map(transport::encode),
        }
    }

    fn region(&self, region: Region) -> Option<&[u8]> {
        match region {
            Region::Preprocessing => Some(&self.preprocessing),
            Region::PublicIo => Some(&self.public_io),
            Region::Proof => Some(&self.proof),
            Region::Commitment => self.trusted_advice_commitment.as_deref(),
        }
    }

    fn with_region(&self, region: Region, bytes: Vec<u8>) -> Self {
        let mut out = self.clone();
        match region {
            Region::Preprocessing => out.preprocessing = bytes,
            Region::PublicIo => out.public_io = bytes,
            Region::Proof => out.proof = bytes,
            Region::Commitment => out.trusted_advice_commitment = Some(bytes),
        }
        out
    }

    /// Decode and verify; `Ok(true)` when the verifier accepts.
    pub fn verify(&self) -> bool {
        let Some(preprocessing) =
            transport::decode::<AkitaVerifierPreprocessing>(&self.preprocessing)
        else {
            stats::count("reject_decode_preprocessing");
            return false;
        };
        let Some(public_io) = transport::decode::<JoltDevice>(&self.public_io) else {
            stats::count("reject_decode_public_io");
            return false;
        };
        let Some(proof) = transport::decode::<Proof>(&self.proof) else {
            stats::count("reject_decode_proof");
            return false;
        };
        let commitment = match &self.trusted_advice_commitment {
            None => None,
            Some(bytes) => match transport::decode::<AkitaCommitment>(bytes) {
                Some(commitment) => Some(commitment),
                None => {
                    stats::count("reject_decode_commitment");
                    return false;
                }
            },
        };
        let accepted = stats::time("verify", || {
            jolt_verifier::verify::<AkitaField, AkitaScheme, AkitaVc, AkitaTranscript>(
                &preprocessing,
                &public_io,
                &proof,
                commitment.as_ref(),
            )
        })
        .is_ok();
        stats::count(if accepted {
            "verify_accept"
        } else {
            "verify_reject"
        });
        accepted
    }

    /// Re-encoding of every region after removing the representation choices
    /// the verifier does not bind: trailing zero bytes of the public inputs
    /// and outputs, which are compared as zero-padded memory, and a
    /// present-or-absent unit `vc_setup` (the Akita build has no vector
    /// commitment). Two bundles with equal semantic forms state the same claim.
    fn semantic(&self) -> Option<[Option<Vec<u8>>; 4]> {
        let mut preprocessing =
            transport::decode::<AkitaVerifierPreprocessing>(&self.preprocessing)?;
        preprocessing.vc_setup = None;
        let mut public_io = transport::decode::<JoltDevice>(&self.public_io)?;
        for bytes in [&mut public_io.inputs, &mut public_io.outputs] {
            while bytes.last() == Some(&0) {
                bytes.pop();
            }
        }
        let mut canonical = self.canonical()?;
        canonical[0] = Some(transport::encode(&preprocessing));
        canonical[1] = Some(transport::encode(&public_io));
        Some(canonical)
    }

    /// Canonical re-encoding of every region, when all decode.
    fn canonical(&self) -> Option<[Option<Vec<u8>>; 4]> {
        Some([
            Some(transport::encode(&transport::decode::<
                AkitaVerifierPreprocessing,
            >(&self.preprocessing)?)),
            Some(transport::encode(&transport::decode::<JoltDevice>(
                &self.public_io,
            )?)),
            Some(transport::encode(&transport::decode::<Proof>(&self.proof)?)),
            match &self.trusted_advice_commitment {
                None => None,
                Some(bytes) => Some(transport::encode(&transport::decode::<AkitaCommitment>(
                    bytes,
                )?)),
            },
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Region {
    Preprocessing,
    PublicIo,
    Proof,
    Commitment,
}

pub fn bundles_dir() -> PathBuf {
    std::env::var_os(BUNDLES_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/bundles"))
}

pub fn load_bundles(dir: &Path) -> Vec<Bundle> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read bundles from {}: {error}", dir.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "bundle"))
        .collect();
    paths.sort();
    let bundles: Vec<Bundle> = paths
        .iter()
        .map(|path| {
            let bytes = std::fs::read(path).expect("read bundle");
            transport::decode(&bytes)
                .unwrap_or_else(|| panic!("{} is not a bundle", path.display()))
        })
        .collect();
    assert!(!bundles.is_empty(), "no bundles in {}", dir.display());
    bundles
}

fn honest() -> &'static [Bundle] {
    static BUNDLES: OnceLock<Vec<Bundle>> = OnceLock::new();
    BUNDLES.get_or_init(|| {
        let bundles = load_bundles(&bundles_dir());
        for bundle in &bundles {
            assert!(
                env::on_large_stack(|| bundle.verify()),
                "honest bundle {} does not verify",
                bundle.name
            );
        }
        bundles
    })
}

/// Apply up to 16 edits to `bytes`; `donor` is another bundle's same region.
fn edit(mut bytes: Vec<u8>, donor: &[u8], reader: &mut Reader<'_>) -> Vec<u8> {
    let count = 1 + reader.u8() % 16;
    for _ in 0..count {
        let op = reader.u8();
        let at = reader.u32() as usize % (bytes.len() + 1);
        let value = reader.u8();
        match op % 8 {
            0 | 1 if at < bytes.len() => bytes[at] ^= value.max(1),
            2 if at < bytes.len() => bytes[at] = value,
            3 => bytes.insert(at, value),
            4 if at < bytes.len() => {
                let end = (at + usize::from(value) + 1).min(bytes.len());
                bytes.drain(at..end);
            }
            5 => bytes.truncate(at),
            6 if at < bytes.len() => {
                // Little-endian length or index fields: add a small delta.
                let delta = i16::from(value as i8);
                let word = bytes[at] as i16 + delta;
                bytes[at] = word as u8;
            }
            7 if !donor.is_empty() => {
                let from = reader.u32() as usize % donor.len();
                let len = (usize::from(value) + 1).min(donor.len() - from);
                let end = (at + len).min(bytes.len());
                bytes.splice(at..end, donor[from..from + len].iter().copied());
            }
            _ => {}
        }
    }
    bytes
}

/// The honest bundle an input edits, the edited region, and the edited bundle.
fn candidate(data: &[u8]) -> Option<(&'static Bundle, Region, Bundle)> {
    let bundles = honest();
    let mut reader = Reader::new(data);
    let bundle = &bundles[usize::from(reader.u8()) % bundles.len()];
    let donor = &bundles[usize::from(reader.u8()) % bundles.len()];
    let region = match reader.u8() % 8 {
        0..=3 => Region::Proof,
        4 | 5 => Region::Preprocessing,
        6 => Region::PublicIo,
        _ => Region::Commitment,
    };
    let original = bundle.region(region)?;
    let edited = edit(
        original.to_vec(),
        donor.region(region).unwrap_or(&[]),
        &mut reader,
    );
    Some((bundle, region, bundle.with_region(region, edited)))
}

fn region_index(region: Region) -> usize {
    match region {
        Region::Preprocessing => 0,
        Region::PublicIo => 1,
        Region::Proof => 2,
        Region::Commitment => 3,
    }
}

/// What an input changes: the edited region's byte differences and the
/// decoded fields that differ from the honest bundle, for triage.
pub fn explain(data: &[u8]) -> String {
    env::init();
    let Some((bundle, region, candidate)) = candidate(data) else {
        return "the input selects an absent region".into();
    };
    let original = bundle.region(region).unwrap_or(&[]);
    let edited = candidate.region(region).unwrap_or(&[]);
    let changed: Vec<usize> = (0..original.len().max(edited.len()))
        .filter(|&at| original.get(at) != edited.get(at))
        .collect();
    let mut out = format!(
        "bundle {} region {region:?}: {} -> {} bytes, {} differing positions (first {:?})\n",
        bundle.name,
        original.len(),
        edited.len(),
        changed.len(),
        &changed[..changed.len().min(16)]
    );
    let accepted = env::on_large_stack(|| candidate.verify());
    out.push_str(&format!("verifier accepts: {accepted}\n"));
    let (honest_a, honest_b, edited_canonical) = (
        bundle.canonical(),
        bundle.canonical(),
        candidate.canonical(),
    );
    out.push_str(&format!(
        "honest re-encoding deterministic: {}; edited re-encodes to honest: {}; honest bytes canonical: {}\n",
        honest_a == honest_b,
        edited_canonical == honest_a,
        honest_a.as_ref().and_then(|canonical| canonical[region_index(region)].as_deref())
            == bundle.region(region),
    ));
    fn json<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Option<String> {
        serde_json::to_string_pretty(&transport::decode::<T>(bytes)?).ok()
    }
    let debug = |bytes: &[u8]| -> Option<String> {
        match region {
            Region::Preprocessing => json::<AkitaVerifierPreprocessing>(bytes),
            Region::PublicIo => json::<JoltDevice>(bytes),
            Region::Proof => json::<Proof>(bytes),
            Region::Commitment => json::<AkitaCommitment>(bytes),
        }
    };
    match (debug(original), debug(edited)) {
        (Some(honest), Some(edited)) => {
            let (honest, edited): (Vec<&str>, Vec<&str>) =
                (honest.lines().collect(), edited.lines().collect());
            out.push_str(&format!(
                "decoded: {} vs {} JSON lines\n",
                honest.len(),
                edited.len()
            ));
            let mut shown = 0;
            for at in 0..honest.len().max(edited.len()) {
                if honest.get(at) != edited.get(at) && shown < 40 {
                    shown += 1;
                    out.push_str(&format!(
                        "  line {at}:\n    honest: {}\n    edited: {}\n",
                        honest.get(at).unwrap_or(&"<none>"),
                        edited.get(at).unwrap_or(&"<none>")
                    ));
                }
            }
        }
        _ => out.push_str("one side does not decode\n"),
    }
    out
}

pub fn run(data: &[u8]) {
    env::init();
    let bundles = honest();
    let Some((bundle, region, candidate)) = candidate(data) else {
        return;
    };
    if !env::on_large_stack(|| candidate.verify()) {
        return;
    }
    // Accepted: the statement and proof must be an honest bundle's, up to
    // the representation choices the verifier does not bind.
    let semantic = candidate.semantic().expect("an accepted bundle decodes");
    let matching = bundles
        .iter()
        .find(|honest| honest.semantic().as_ref() == Some(&semantic))
        .unwrap_or_else(|| {
            panic!(
                "soundness: verifier accepted a non-honest {region:?} edit of bundle {}",
                bundle.name
            )
        });
    if candidate.region(region) != matching.region(region) {
        stats::count(if candidate.canonical() == matching.canonical() {
            "noncanonical_encoding_accepted"
        } else {
            "equivalent_statement_accepted"
        });
        return;
    }
    stats::count("identity_accepted");
}

/// A no-op edit of every region of every bundle, plus one byte flip each.
pub fn seeds() -> Vec<(String, Vec<u8>)> {
    let mut seeds = Vec::new();
    for bundle in 0..8u8 {
        for (region_name, region) in [
            ("proof", 0u8),
            ("preprocessing", 4),
            ("io", 6),
            ("commitment", 7),
        ] {
            let mut bytes = vec![bundle, bundle.wrapping_add(1), region, 0];
            // One edit: op 6 (small delta) at offset 0 with delta 0.
            bytes.extend([6, 0, 0, 0, 0, 0]);
            seeds.push((
                format!("bundle{bundle}-{region_name}-identity"),
                bytes.clone(),
            ));
            bytes[4] = 0;
            bytes[5..9].copy_from_slice(&64u32.to_le_bytes());
            bytes[9] = 1;
            seeds.push((format!("bundle{bundle}-{region_name}-flip64"), bytes));
        }
    }
    seeds
}

/// Prove the program lane's seeds for these guests and write one bundle each.
pub fn write_bundles(out: &Path) -> Result<(), String> {
    use crate::targets::program;
    std::fs::create_dir_all(out).map_err(|e| format!("create {}: {e}", out.display()))?;
    let wanted = [
        "muldiv-default",
        "muldiv-committed2",
        "muldiv-k256-address-first",
        "muldiv-four-chunks",
        "advice-consumer-default",
        "interp-default",
        "interp-advice-large-default",
    ];
    for (name, bytes) in program::seeds() {
        if !wanted.contains(&name.as_str()) {
            continue;
        }
        let (guest, options, args) = program::decode(&bytes);
        let proved = env::on_large_stack(|| program::prove(guest, &args, &options))
            .map_err(|failure| format!("{name}: {failure}"))?
            .ok_or_else(|| format!("{name}: skipped (trace budget or unlisted K=256 group)"))?;
        let bundle = Bundle::new(
            &name,
            &proved.preprocessing.verifier,
            &proved.public_io,
            &proved.proof,
            proved.trusted_advice_commitment.as_ref(),
        );
        std::fs::write(
            out.join(format!("{name}.bundle")),
            transport::encode(&bundle),
        )
        .map_err(|e| e.to_string())?;
        eprintln!(
            "{name}: proof {} B, preprocessing {} B",
            bundle.proof.len(),
            bundle.preprocessing.len()
        );
    }
    Ok(())
}
