use crate::field::fp128::JoltFp128;
use crate::transcripts::Transcript as JoltTranscript;
use akita_field::CanonicalField;
use akita_field::FieldCore;
use akita_field::Prime128OffsetA7F7;
use akita_field::RandomSampling;
use akita_prover::AkitaProverSetup as UpstreamAkitaProverSetup;
use akita_serialization::{
    AkitaDeserialize, AkitaSerialize, Compress as AkitaCompress, Valid as AkitaValid,
    Validate as AkitaValidate,
};
use akita_transcript::Transcript as AkitaTranscript;
use akita_types::{
    AkitaBatchedProof, AkitaBatchedProofShape, AkitaExpandedSetup,
    AkitaVerifierSetup as UpstreamAkitaVerifierSetup, RingCommitment,
};
use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use std::io::{Read, Write};
use std::sync::Arc;

pub type Fp128 = Prime128OffsetA7F7;

#[inline]
pub fn jolt_to_akita(f: &JoltFp128) -> Fp128 {
    // SAFETY: JoltFp128 is repr(transparent) over Prime128OffsetA7F7.
    unsafe { std::mem::transmute_copy(f) }
}

#[inline]
#[allow(dead_code)]
pub fn akita_to_jolt(f: &Fp128) -> JoltFp128 {
    // SAFETY: JoltFp128 is repr(transparent) over Prime128OffsetA7F7.
    unsafe { std::mem::transmute_copy(f) }
}

struct TranscriptSyncTarget<T: JoltTranscript> {
    ptr: *mut T,
}

unsafe impl<T: JoltTranscript> Send for TranscriptSyncTarget<T> {}
unsafe impl<T: JoltTranscript> Sync for TranscriptSyncTarget<T> {}

pub type AkitaProof<F> = AkitaBatchedProof<F, F>;
pub type AkitaVerifierSetup<F> = UpstreamAkitaVerifierSetup<F>;
pub type AkitaProverSetup<F, const D: usize> = UpstreamAkitaProverSetup<F, D>;

/// Bridge adapter: wraps a Jolt transcript pointer and implements Akita's Transcript trait.
///
/// Uses a raw pointer internally because Akita's `Transcript` trait requires `'static`,
/// but we need to borrow a Jolt transcript that has a limited lifetime. The adapter is
/// always used in a strictly scoped manner within a single prove/verify call.
pub struct JoltToAkitaTranscript<T: JoltTranscript> {
    state: T,
    sync_target: Option<Arc<TranscriptSyncTarget<T>>>,
}

unsafe impl<T: JoltTranscript> Send for JoltToAkitaTranscript<T> {}
unsafe impl<T: JoltTranscript> Sync for JoltToAkitaTranscript<T> {}

impl<T: JoltTranscript> JoltToAkitaTranscript<T> {
    pub fn new(transcript: &mut T) -> Self {
        Self {
            state: transcript.clone(),
            sync_target: Some(Arc::new(TranscriptSyncTarget {
                ptr: transcript as *mut T,
            })),
        }
    }

    fn inner(&mut self) -> &mut T {
        &mut self.state
    }

    #[inline]
    fn absorb_label(&mut self, label: &[u8]) {
        self.inner().append_bytes(b"akita_label", label);
    }
}

impl<T: JoltTranscript> Clone for JoltToAkitaTranscript<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            sync_target: self.sync_target.clone(),
        }
    }
}

impl<T: JoltTranscript> Drop for JoltToAkitaTranscript<T> {
    fn drop(&mut self) {
        if let Some(target) = &self.sync_target {
            if Arc::strong_count(target) == 1 {
                // SAFETY: `sync_target` originates from `new(&mut T)` and remains valid for the
                // scoped lifetime of all adapter clones. Only the last surviving clone syncs back,
                // which preserves Akita's clone-and-commit transcript pattern without letting
                // speculative clones overwrite the caller transcript.
                unsafe {
                    *target.ptr = self.state.clone();
                }
            }
        }
    }
}

impl<T: JoltTranscript> AkitaTranscript<Fp128> for JoltToAkitaTranscript<T> {
    fn new(_domain_label: &[u8]) -> Self {
        unimplemented!("use JoltToAkitaTranscript::new(transcript) to wrap an existing transcript")
    }

    fn bind_instance_bytes(&mut self, instance_bytes: &[u8]) {
        self.inner().append_bytes(b"akita_instance", instance_bytes);
    }

    fn append_bytes(&mut self, _label: &[u8], bytes: &[u8]) {
        self.absorb_label(_label);
        self.inner().append_bytes(b"akita_bytes", bytes);
    }

    fn append_field(&mut self, _label: &[u8], x: &Fp128) {
        self.absorb_label(_label);
        let val = x.to_canonical_u128();
        self.inner()
            .append_bytes(b"akita_field", &val.to_le_bytes());
    }

    fn append_serde<S: AkitaSerialize>(&mut self, _label: &[u8], s: &S) {
        self.absorb_label(_label);
        let mut buf = Vec::with_capacity(s.serialized_size(AkitaCompress::No));
        s.serialize_uncompressed(&mut buf)
            .expect("AkitaSerialize should not fail");
        self.inner().append_bytes(b"akita_serde", &buf);
    }

    fn challenge_scalar(&mut self, _label: &[u8]) -> Fp128 {
        self.absorb_label(_label);
        let jolt_challenge: JoltFp128 = self.inner().challenge_scalar();
        jolt_to_akita(&jolt_challenge)
    }

    fn challenge_bytes(&mut self, _label: &[u8], len: usize) -> Vec<u8> {
        self.absorb_label(_label);
        let mut out = Vec::with_capacity(len);
        while out.len() < len {
            out.extend_from_slice(&self.inner().challenge_u128().to_le_bytes());
        }
        out.truncate(len);
        out
    }
}

/// Newtype wrapper that bridges Akita's native `AkitaSerialize`/`AkitaDeserialize`
/// to arkworks' `CanonicalSerialize`/`CanonicalDeserialize`, which Jolt's
/// `CommitmentScheme` associated-type bounds require.
///
/// The two serialization frameworks differ in one important way: Akita's
/// `AkitaDeserialize` carries an associated `Context` describing shape information
/// that the byte stream alone cannot recover (e.g. an `AkitaBatchedProof` needs an
/// `AkitaBatchedProofShape`), whereas arkworks' `deserialize_with_mode` is
/// context-free. Because the context differs per type, there is no single blanket
/// bridge; instead each wrapped Akita type gets its own impl below:
///
/// - Self-describing types (`RingCommitment`, `AkitaVerifierSetup`) delegate
///   directly to the native codec with `Context = ()`.
/// - `AkitaProof` is encoded self-describingly by writing its
///   `AkitaBatchedProofShape` (itself `Context = ()`) as a prefix, then reading it
///   back to supply the context on deserialize.
/// - `AkitaProverSetup` persists only its derived `expanded` setup and is rebuilt
///   via `from_seed_validated_expanded` on deserialize.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArkBridge<T: Send + Sync>(pub T);

impl<T: Default + Send + Sync> Default for ArkBridge<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T: Send + Sync> Valid for ArkBridge<T> {
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

fn ark_to_akita_compress(c: Compress) -> AkitaCompress {
    match c {
        Compress::Yes => AkitaCompress::Yes,
        Compress::No => AkitaCompress::No,
    }
}

fn ark_to_akita_validate(v: Validate) -> AkitaValidate {
    match v {
        Validate::Yes => AkitaValidate::Yes,
        Validate::No => AkitaValidate::No,
    }
}

/// Map an Akita serialization error into arkworks' error channel.
fn akita_err_to_ark(e: akita_serialization::SerializationError) -> SerializationError {
    SerializationError::IoError(std::io::Error::other(e.to_string()))
}

/// Helper to bridge an `AkitaSerialize` type's serializer through arkworks'
/// `CanonicalSerialize` error channel.
fn akita_serialize_to_ark<W: Write, T: AkitaSerialize>(
    value: &T,
    writer: W,
    compress: Compress,
) -> Result<(), SerializationError> {
    value
        .serialize_with_mode(writer, ark_to_akita_compress(compress))
        .map_err(akita_err_to_ark)
}

/// Helper to bridge a self-describing (`Context = ()`) `AkitaDeserialize` type
/// through arkworks' `CanonicalDeserialize` error channel.
fn akita_deserialize_from_ark<R: Read, T: AkitaDeserialize<Context = ()>>(
    reader: R,
    compress: Compress,
    validate: Validate,
) -> Result<T, SerializationError> {
    T::deserialize_with_mode(
        reader,
        ark_to_akita_compress(compress),
        ark_to_akita_validate(validate),
        &(),
    )
    .map_err(akita_err_to_ark)
}

macro_rules! impl_ark_serde_via_akita_context_free {
    ($ty:ty, [$($bound:tt)*]) => {
        impl<F: FieldCore + AkitaSerialize $($bound)*> CanonicalSerialize for ArkBridge<$ty> {
            fn serialize_with_mode<W: Write>(
                &self,
                writer: W,
                compress: Compress,
            ) -> Result<(), SerializationError> {
                akita_serialize_to_ark(&self.0, writer, compress)
            }
            fn serialized_size(&self, compress: Compress) -> usize {
                self.0.serialized_size(ark_to_akita_compress(compress))
            }
        }

        impl<F: FieldCore + AkitaDeserialize<Context = ()> + AkitaValid $($bound)*>
            CanonicalDeserialize for ArkBridge<$ty>
        {
            fn deserialize_with_mode<R: Read>(
                reader: R,
                compress: Compress,
                validate: Validate,
            ) -> Result<Self, SerializationError> {
                Ok(ArkBridge(akita_deserialize_from_ark::<R, $ty>(
                    reader, compress, validate,
                )?))
            }
        }
    };
}

impl_ark_serde_via_akita_context_free!(AkitaVerifierSetup<F>, [+ RandomSampling]);

/// `AkitaProof` deserialization needs an `AkitaBatchedProofShape`, which arkworks'
/// context-free codec cannot supply. We make the wire encoding self-describing by
/// writing the proof's shape (`Context = ()`) as a prefix, then reading it back to
/// drive the shape-aware proof decoder.
impl<F: CanonicalField + FieldCore + AkitaSerialize> CanonicalSerialize
    for ArkBridge<AkitaProof<F>>
{
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        let akita_compress = ark_to_akita_compress(compress);
        self.0
            .shape()
            .serialize_with_mode(&mut writer, akita_compress)
            .map_err(akita_err_to_ark)?;
        self.0
            .serialize_with_mode(&mut writer, akita_compress)
            .map_err(akita_err_to_ark)
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        let akita_compress = ark_to_akita_compress(compress);
        self.0.shape().serialized_size(akita_compress) + self.0.serialized_size(akita_compress)
    }
}

impl<F: CanonicalField + FieldCore + AkitaDeserialize<Context = ()> + AkitaValid>
    CanonicalDeserialize for ArkBridge<AkitaProof<F>>
{
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let akita_compress = ark_to_akita_compress(compress);
        let akita_validate = ark_to_akita_validate(validate);
        let shape = AkitaBatchedProofShape::deserialize_with_mode(
            &mut reader,
            akita_compress,
            akita_validate,
            &(),
        )
        .map_err(akita_err_to_ark)?;
        let proof = AkitaProof::<F>::deserialize_with_mode(
            &mut reader,
            akita_compress,
            akita_validate,
            &shape,
        )
        .map_err(akita_err_to_ark)?;
        Ok(ArkBridge(proof))
    }
}

impl<F: FieldCore + AkitaSerialize, const D: usize> CanonicalSerialize
    for ArkBridge<RingCommitment<F, D>>
{
    fn serialize_with_mode<W: Write>(
        &self,
        writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        akita_serialize_to_ark(&self.0, writer, compress)
    }
    fn serialized_size(&self, compress: Compress) -> usize {
        self.0.serialized_size(ark_to_akita_compress(compress))
    }
}

impl<F: FieldCore + AkitaDeserialize<Context = ()> + AkitaValid, const D: usize>
    CanonicalDeserialize for ArkBridge<RingCommitment<F, D>>
{
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        Ok(ArkBridge(akita_deserialize_from_ark::<
            R,
            RingCommitment<F, D>,
        >(reader, compress, validate)?))
    }
}

/// `AkitaProverSetup` intentionally does not implement `AkitaSerialize` upstream
/// (its NTT cache is derived). Delegate serialization to the inner expanded
/// setup so Jolt's `CommitmentScheme::ProverSetup` bound is still satisfied.
impl<F: FieldCore + AkitaSerialize, const D: usize> CanonicalSerialize
    for ArkBridge<AkitaProverSetup<F, D>>
{
    fn serialize_with_mode<W: Write>(
        &self,
        writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        akita_serialize_to_ark(&*self.0.expanded, writer, compress)
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        self.0
            .expanded
            .serialized_size(ark_to_akita_compress(compress))
    }
}

/// Reconstruct the prover setup from its persisted `expanded` setup. The derived
/// prefix-slot registry is rebuilt by `from_seed_validated_expanded`, mirroring the
/// serialize side which only persists `expanded`.
impl<
        F: FieldCore + CanonicalField + RandomSampling + AkitaValid + AkitaDeserialize<Context = ()>,
        const D: usize,
    > CanonicalDeserialize for ArkBridge<AkitaProverSetup<F, D>>
{
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let expanded =
            akita_deserialize_from_ark::<R, AkitaExpandedSetup<F>>(reader, compress, validate)?;
        let setup = UpstreamAkitaProverSetup::<F, D>::from_seed_validated_expanded(expanded)
            .map_err(|e| SerializationError::IoError(std::io::Error::other(e.to_string())))?;
        Ok(ArkBridge(setup))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::JoltField;
    use akita_algebra::CyclotomicRing;

    const D: usize = 64;

    fn sample_ring(seed: u64) -> CyclotomicRing<Fp128, D> {
        let coeffs = std::array::from_fn(|i| {
            jolt_to_akita(&JoltFp128::from_u64(
                seed.wrapping_add(i as u64).wrapping_add(1),
            ))
        });
        CyclotomicRing::from_coefficients(coeffs)
    }

    fn roundtrip_ring_commitment(compress: Compress) {
        let commitment = RingCommitment::<Fp128, D> {
            u: vec![sample_ring(7), sample_ring(101), sample_ring(255)],
        };
        let original = ArkBridge(commitment);

        let mut bytes = Vec::new();
        original.serialize_with_mode(&mut bytes, compress).unwrap();
        assert_eq!(
            bytes.len(),
            original.serialized_size(compress),
            "serialized_size must match emitted byte length"
        );

        let decoded = ArkBridge::<RingCommitment<Fp128, D>>::deserialize_with_mode(
            &bytes[..],
            compress,
            Validate::Yes,
        )
        .expect("ArkBridge round trip should succeed");

        assert_eq!(original, decoded, "ArkBridge round trip must be lossless");
    }

    #[test]
    fn ark_bridge_ring_commitment_round_trips_compressed() {
        roundtrip_ring_commitment(Compress::Yes);
    }

    #[test]
    fn ark_bridge_ring_commitment_round_trips_uncompressed() {
        roundtrip_ring_commitment(Compress::No);
    }
}
