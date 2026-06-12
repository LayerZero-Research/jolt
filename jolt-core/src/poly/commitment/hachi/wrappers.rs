use crate::field::fp128::JoltFp128;
use crate::transcripts::Transcript as JoltTranscript;
use akita_field::FieldCore;
use akita_field::Prime128OffsetA7F7;
use akita_prover::AkitaProverSetup;
use akita_serialization::{AkitaSerialize, Compress as AkitaCompress};
use akita_transcript::Transcript as AkitaTranscript;
use akita_types::{AkitaBatchedProof, AkitaVerifierSetup, RingCommitment};
use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use std::io::{Read, Write};
use std::sync::Arc;

pub type Fp128 = Prime128OffsetA7F7;

#[inline]
pub fn jolt_to_hachi(f: &JoltFp128) -> Fp128 {
    // SAFETY: JoltFp128 is repr(transparent) over Prime128OffsetA7F7.
    unsafe { std::mem::transmute_copy(f) }
}

#[inline]
#[allow(dead_code)]
pub fn hachi_to_jolt(f: &Fp128) -> JoltFp128 {
    // SAFETY: JoltFp128 is repr(transparent) over Prime128OffsetA7F7.
    unsafe { std::mem::transmute_copy(f) }
}

struct TranscriptSyncTarget<T: JoltTranscript> {
    ptr: *mut T,
}

unsafe impl<T: JoltTranscript> Send for TranscriptSyncTarget<T> {}
unsafe impl<T: JoltTranscript> Sync for TranscriptSyncTarget<T> {}

pub type HachiProof<F> = AkitaBatchedProof<F, F>;
pub type HachiVerifierSetup<F> = AkitaVerifierSetup<F>;
pub type HachiProverSetup<F, const D: usize> = AkitaProverSetup<F, D>;

/// Bridge adapter: wraps a Jolt transcript pointer and implements Akita's Transcript trait.
///
/// Uses a raw pointer internally because Akita's `Transcript` trait requires `'static`,
/// but we need to borrow a Jolt transcript that has a limited lifetime. The adapter is
/// always used in a strictly scoped manner within a single prove/verify call.
pub struct JoltToHachiTranscript<T: JoltTranscript> {
    state: T,
    sync_target: Option<Arc<TranscriptSyncTarget<T>>>,
}

unsafe impl<T: JoltTranscript> Send for JoltToHachiTranscript<T> {}
unsafe impl<T: JoltTranscript> Sync for JoltToHachiTranscript<T> {}

impl<T: JoltTranscript> JoltToHachiTranscript<T> {
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
        self.inner().append_bytes(b"hachi_label", label);
    }
}

impl<T: JoltTranscript> Clone for JoltToHachiTranscript<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            sync_target: self.sync_target.clone(),
        }
    }
}

impl<T: JoltTranscript> Drop for JoltToHachiTranscript<T> {
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

impl<T: JoltTranscript> AkitaTranscript<Fp128> for JoltToHachiTranscript<T> {
    fn new(_domain_label: &[u8]) -> Self {
        unimplemented!("use JoltToHachiTranscript::new(transcript) to wrap an existing transcript")
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
        jolt_to_hachi(&jolt_challenge)
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

/// Newtype wrapper that provides arkworks `CanonicalSerialize`/`CanonicalDeserialize`
/// for Akita types. These are stub implementations: the actual serialization path for
/// Akita types uses `AkitaSerialize`/`AkitaDeserialize`. The arkworks traits exist
/// solely to satisfy Jolt's `CommitmentScheme` associated type bounds.
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

/// Helper to bridge an `AkitaSerialize` type's serializer through arkworks'
/// `CanonicalSerialize` error channel.
fn akita_serialize_to_ark<W: Write, T: AkitaSerialize>(
    value: &T,
    writer: W,
    compress: Compress,
) -> Result<(), SerializationError> {
    value
        .serialize_with_mode(writer, ark_to_akita_compress(compress))
        .map_err(|e| SerializationError::IoError(std::io::Error::other(e.to_string())))
}

impl<T: Send + Sync> CanonicalDeserialize for ArkBridge<T> {
    fn deserialize_with_mode<R: Read>(
        _reader: R,
        _compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        unimplemented!("Akita types use AkitaDeserialize, not CanonicalDeserialize")
    }
}

macro_rules! impl_ark_serialize_via_akita {
    ($ty:ty) => {
        impl<F: FieldCore + AkitaSerialize> CanonicalSerialize for ArkBridge<$ty> {
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
    };
}

impl_ark_serialize_via_akita!(HachiProof<F>);
impl_ark_serialize_via_akita!(HachiVerifierSetup<F>);

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

/// `AkitaProverSetup` intentionally does not implement `AkitaSerialize` upstream
/// (its NTT cache is derived). Delegate serialization to the inner expanded
/// setup so Jolt's `CommitmentScheme::ProverSetup` bound is still satisfied.
impl<F: FieldCore + AkitaSerialize, const D: usize> CanonicalSerialize
    for ArkBridge<HachiProverSetup<F, D>>
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
