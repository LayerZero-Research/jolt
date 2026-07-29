#[cfg(all(not(feature = "zk"), not(feature = "akita")))]
use std::collections::BTreeMap;
use std::io::{Read, Write};

use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use num::FromPrimitive;
use strum::EnumCount;

#[cfg(all(not(feature = "zk"), not(feature = "akita")))]
use crate::poly::opening_proof::{OpeningPoint, Openings};
#[cfg(feature = "zk")]
use crate::subprotocols::blindfold::BlindFoldProof;
#[cfg(not(feature = "akita"))]
use crate::{
    curve::JoltCurve, field::JoltField, poly::commitment::commitment_scheme::CommitmentScheme,
};
use crate::{
    poly::{
        commitment::dory::DoryLayout,
        opening_proof::{OpeningId, PolynomialId, SumcheckId},
    },
    zkvm::{
        instruction::{CircuitFlags, InstructionFlags},
        witness::{CommittedPolynomial, VirtualPolynomial},
    },
};
#[cfg(not(feature = "akita"))]
use crate::{
    subprotocols::{
        sumcheck::SumcheckInstanceProof, univariate_skip::UniSkipFirstRoundProofVariant,
    },
    transcripts::Transcript,
    zkvm::config::{OneHotConfig, ReadWriteConfig},
};

/// The base (Dory) prove path's proof payload. The akita path emits the
/// verifier-native `JoltProof` directly and never builds these parts.
#[cfg(not(feature = "akita"))]
pub(crate) struct JoltProofParts<
    F: JoltField,
    C: JoltCurve<F = F>,
    PCS: CommitmentScheme<Field = F>,
    FS: Transcript,
> {
    pub commitments: Vec<PCS::Commitment>,
    pub stage1_uni_skip_first_round_proof: UniSkipFirstRoundProofVariant<F, C, FS>,
    pub stage1_sumcheck_proof: SumcheckInstanceProof<F, C, FS>,
    pub stage2_uni_skip_first_round_proof: UniSkipFirstRoundProofVariant<F, C, FS>,
    pub stage2_sumcheck_proof: SumcheckInstanceProof<F, C, FS>,
    pub stage3_sumcheck_proof: SumcheckInstanceProof<F, C, FS>,
    pub stage4_sumcheck_proof: SumcheckInstanceProof<F, C, FS>,
    pub stage5_sumcheck_proof: SumcheckInstanceProof<F, C, FS>,
    pub stage6a_sumcheck_proof: SumcheckInstanceProof<F, C, FS>,
    pub stage6b_sumcheck_proof: SumcheckInstanceProof<F, C, FS>,
    pub stage7_sumcheck_proof: SumcheckInstanceProof<F, C, FS>,
    #[cfg(feature = "zk")]
    pub blindfold_proof: BlindFoldProof<F, C>,
    pub joint_opening_proof: PCS::Proof,
    pub untrusted_advice_commitment: Option<PCS::Commitment>,
    #[cfg(not(feature = "zk"))]
    pub opening_claims: ProverOpeningClaims<F>,
    pub trace_length: usize,
    pub ram_K: usize,
    pub rw_config: ReadWriteConfig,
    pub one_hot_config: OneHotConfig,
    pub dory_layout: DoryLayout,
}

#[cfg(all(not(feature = "zk"), not(feature = "akita")))]
pub(crate) struct ProverOpeningClaims<F: JoltField>(pub Openings<F>);

#[cfg(all(not(feature = "zk"), not(feature = "akita")))]
impl<F: JoltField> CanonicalSerialize for ProverOpeningClaims<F> {
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        self.0.len().serialize_with_mode(&mut writer, compress)?;
        for (key, (_opening_point, claim)) in self.0.iter() {
            key.serialize_with_mode(&mut writer, compress)?;
            claim.serialize_with_mode(&mut writer, compress)?;
        }
        Ok(())
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        let mut size = self.0.len().serialized_size(compress);
        for (key, (_opening_point, claim)) in self.0.iter() {
            size += key.serialized_size(compress);
            size += claim.serialized_size(compress);
        }
        size
    }
}

#[cfg(all(not(feature = "zk"), not(feature = "akita")))]
impl<F: JoltField> Valid for ProverOpeningClaims<F> {
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

#[cfg(all(not(feature = "zk"), not(feature = "akita")))]
impl<F: JoltField> CanonicalDeserialize for ProverOpeningClaims<F> {
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let size = usize::deserialize_with_mode(&mut reader, compress, validate)?;
        let mut claims = BTreeMap::new();
        for _ in 0..size {
            let key = OpeningId::deserialize_with_mode(&mut reader, compress, validate)?;
            let claim = F::deserialize_with_mode(&mut reader, compress, validate)?;
            claims.insert(key, (OpeningPoint::default(), claim));
        }
        Ok(ProverOpeningClaims(claims))
    }
}

impl CanonicalSerialize for DoryLayout {
    fn serialize_with_mode<W: Write>(
        &self,
        writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        u8::from(*self).serialize_with_mode(writer, compress)
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        u8::from(*self).serialized_size(compress)
    }
}

impl Valid for DoryLayout {
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

impl CanonicalDeserialize for DoryLayout {
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let value = u8::deserialize_with_mode(reader, compress, validate)?;
        if value > 1 {
            return Err(SerializationError::InvalidData);
        }
        Ok(DoryLayout::from(value))
    }
}

// Compact encoding for OpeningId:
// Each variant uses a fused byte = BASE + sumcheck_id (1 byte total for advice, 2 bytes for committed/virtual)
// - [0, NUM_SUMCHECKS) = UntrustedAdvice(sumcheck_id)
// - [NUM_SUMCHECKS, 2*NUM_SUMCHECKS) = TrustedAdvice(sumcheck_id)
// - [2*NUM_SUMCHECKS, 3*NUM_SUMCHECKS) + poly_index = Committed(poly, sumcheck_id)
// - [3*NUM_SUMCHECKS, 4*NUM_SUMCHECKS) + poly_index = Virtual(poly, sumcheck_id)
const OPENING_ID_UNTRUSTED_ADVICE_BASE: u8 = 0;
const OPENING_ID_TRUSTED_ADVICE_BASE: u8 =
    OPENING_ID_UNTRUSTED_ADVICE_BASE + SumcheckId::COUNT as u8;
const OPENING_ID_COMMITTED_BASE: u8 = OPENING_ID_TRUSTED_ADVICE_BASE + SumcheckId::COUNT as u8;
const OPENING_ID_VIRTUAL_BASE: u8 = OPENING_ID_COMMITTED_BASE + SumcheckId::COUNT as u8;

impl CanonicalSerialize for OpeningId {
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        match self {
            OpeningId::UntrustedAdvice(sumcheck_id) => {
                let fused = OPENING_ID_UNTRUSTED_ADVICE_BASE + (*sumcheck_id as u8);
                fused.serialize_with_mode(&mut writer, compress)
            }
            OpeningId::TrustedAdvice(sumcheck_id) => {
                let fused = OPENING_ID_TRUSTED_ADVICE_BASE + (*sumcheck_id as u8);
                fused.serialize_with_mode(&mut writer, compress)
            }
            OpeningId::Polynomial(PolynomialId::Committed(committed_polynomial), sumcheck_id) => {
                let fused = OPENING_ID_COMMITTED_BASE + (*sumcheck_id as u8);
                fused.serialize_with_mode(&mut writer, compress)?;
                committed_polynomial.serialize_with_mode(&mut writer, compress)
            }
            OpeningId::Polynomial(PolynomialId::Virtual(virtual_polynomial), sumcheck_id) => {
                let fused = OPENING_ID_VIRTUAL_BASE + (*sumcheck_id as u8);
                fused.serialize_with_mode(&mut writer, compress)?;
                virtual_polynomial.serialize_with_mode(&mut writer, compress)
            }
        }
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        match self {
            OpeningId::UntrustedAdvice(_) | OpeningId::TrustedAdvice(_) => 1,
            OpeningId::Polynomial(PolynomialId::Committed(committed_polynomial), _) => {
                1 + committed_polynomial.serialized_size(compress)
            }
            OpeningId::Polynomial(PolynomialId::Virtual(virtual_polynomial), _) => {
                1 + virtual_polynomial.serialized_size(compress)
            }
        }
    }
}

impl Valid for OpeningId {
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

impl CanonicalDeserialize for OpeningId {
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        let fused = u8::deserialize_with_mode(&mut reader, compress, validate)?;
        match fused {
            _ if fused < OPENING_ID_TRUSTED_ADVICE_BASE => {
                let sumcheck_id = fused - OPENING_ID_UNTRUSTED_ADVICE_BASE;
                Ok(OpeningId::UntrustedAdvice(
                    SumcheckId::from_u8(sumcheck_id).ok_or(SerializationError::InvalidData)?,
                ))
            }
            _ if fused < OPENING_ID_COMMITTED_BASE => {
                let sumcheck_id = fused - OPENING_ID_TRUSTED_ADVICE_BASE;
                Ok(OpeningId::TrustedAdvice(
                    SumcheckId::from_u8(sumcheck_id).ok_or(SerializationError::InvalidData)?,
                ))
            }
            _ if fused < OPENING_ID_VIRTUAL_BASE => {
                let sumcheck_id = fused - OPENING_ID_COMMITTED_BASE;
                let polynomial =
                    CommittedPolynomial::deserialize_with_mode(&mut reader, compress, validate)?;
                Ok(OpeningId::committed(
                    polynomial,
                    SumcheckId::from_u8(sumcheck_id).ok_or(SerializationError::InvalidData)?,
                ))
            }
            _ => {
                let sumcheck_id = fused - OPENING_ID_VIRTUAL_BASE;
                let polynomial =
                    VirtualPolynomial::deserialize_with_mode(&mut reader, compress, validate)?;
                Ok(OpeningId::virt(
                    polynomial,
                    SumcheckId::from_u8(sumcheck_id).ok_or(SerializationError::InvalidData)?,
                ))
            }
        }
    }
}

/// Wire tags for [`CommittedPolynomial`]. The encode, decode and size paths all
/// read this one table, so a tag can never mean two things.
mod committed_tag {
    pub const RD_INC: u8 = 0;
    pub const RAM_INC: u8 = 1;
    pub const INSTRUCTION_RA: u8 = 2;
    pub const BYTECODE_RA: u8 = 3;
    pub const RAM_RA: u8 = 4;
    pub const TRUSTED_ADVICE: u8 = 5;
    pub const UNTRUSTED_ADVICE: u8 = 6;
    pub const BYTECODE_CHUNK: u8 = 7;
    pub const PROGRAM_IMAGE_INIT: u8 = 8;
    pub const UNSIGNED_INC_CHUNK: u8 = 9;
    pub const UNSIGNED_INC_MSB: u8 = 10;
    pub const BYTECODE_REGISTER_SELECTOR: u8 = 11;
    pub const BYTECODE_CIRCUIT_FLAG: u8 = 12;
    pub const BYTECODE_INSTRUCTION_FLAG: u8 = 13;
    pub const BYTECODE_LOOKUP_SELECTOR: u8 = 14;
    pub const BYTECODE_RAF_FLAG: u8 = 15;
    pub const BYTECODE_UNEXPANDED_PC_BYTES: u8 = 16;
    pub const BYTECODE_IMM_BYTES: u8 = 17;
    pub const PROGRAM_IMAGE_BYTES: u8 = 18;
}

/// A `usize` index that does not fit the one-byte wire field.
fn index_out_of_range() -> SerializationError {
    SerializationError::InvalidData
}

impl CommittedPolynomial {
    /// The variant's wire tag. Exhaustive, so a new variant is a compile error.
    const fn wire_tag(&self) -> u8 {
        use committed_tag as tag;
        match self {
            Self::RdInc => tag::RD_INC,
            Self::RamInc => tag::RAM_INC,
            Self::InstructionRa(_) => tag::INSTRUCTION_RA,
            Self::BytecodeRa(_) => tag::BYTECODE_RA,
            Self::RamRa(_) => tag::RAM_RA,
            Self::TrustedAdvice => tag::TRUSTED_ADVICE,
            Self::UntrustedAdvice => tag::UNTRUSTED_ADVICE,
            Self::BytecodeChunk(_) => tag::BYTECODE_CHUNK,
            Self::ProgramImageInit => tag::PROGRAM_IMAGE_INIT,
            Self::UnsignedIncChunk(_) => tag::UNSIGNED_INC_CHUNK,
            Self::UnsignedIncMsb => tag::UNSIGNED_INC_MSB,
            Self::BytecodeRegisterSelector(..) => tag::BYTECODE_REGISTER_SELECTOR,
            Self::BytecodeCircuitFlag(..) => tag::BYTECODE_CIRCUIT_FLAG,
            Self::BytecodeInstructionFlag(..) => tag::BYTECODE_INSTRUCTION_FLAG,
            Self::BytecodeLookupSelector(_) => tag::BYTECODE_LOOKUP_SELECTOR,
            Self::BytecodeRafFlag(_) => tag::BYTECODE_RAF_FLAG,
            Self::BytecodeUnexpandedPcBytes(_) => tag::BYTECODE_UNEXPANDED_PC_BYTES,
            Self::BytecodeImmBytes(_) => tag::BYTECODE_IMM_BYTES,
            Self::ProgramImageBytes => tag::PROGRAM_IMAGE_BYTES,
        }
    }

    /// The index fields written after the tag, in wire order. The returned
    /// length is the payload byte count `serialized_size` reports.
    fn wire_fields(&self) -> ([usize; 2], usize) {
        match self {
            Self::RdInc
            | Self::RamInc
            | Self::TrustedAdvice
            | Self::UntrustedAdvice
            | Self::ProgramImageInit
            | Self::UnsignedIncMsb
            | Self::ProgramImageBytes => ([0, 0], 0),
            Self::InstructionRa(i)
            | Self::BytecodeRa(i)
            | Self::RamRa(i)
            | Self::BytecodeChunk(i)
            | Self::UnsignedIncChunk(i)
            | Self::BytecodeLookupSelector(i)
            | Self::BytecodeRafFlag(i)
            | Self::BytecodeUnexpandedPcBytes(i)
            | Self::BytecodeImmBytes(i) => ([*i, 0], 1),
            Self::BytecodeRegisterSelector(first, second)
            | Self::BytecodeCircuitFlag(first, second)
            | Self::BytecodeInstructionFlag(first, second) => ([*first, *second], 2),
        }
    }
}

impl CanonicalSerialize for CommittedPolynomial {
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        self.wire_tag().serialize_with_mode(&mut writer, compress)?;
        let (fields, count) = self.wire_fields();
        for field in &fields[..count] {
            u8::try_from(*field)
                .map_err(|_| index_out_of_range())?
                .serialize_with_mode(&mut writer, compress)?;
        }
        Ok(())
    }

    fn serialized_size(&self, compress: Compress) -> usize {
        let (_, count) = self.wire_fields();
        self.wire_tag().serialized_size(compress) + count
    }
}

impl Valid for CommittedPolynomial {
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

impl CanonicalDeserialize for CommittedPolynomial {
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        use committed_tag as tag;
        let tag_byte = u8::deserialize_with_mode(&mut reader, compress, validate)?;
        let field = |reader: &mut R| -> Result<usize, SerializationError> {
            u8::deserialize_with_mode(reader, compress, validate).map(usize::from)
        };
        Ok(match tag_byte {
            tag::RD_INC => Self::RdInc,
            tag::RAM_INC => Self::RamInc,
            tag::INSTRUCTION_RA => Self::InstructionRa(field(&mut reader)?),
            tag::BYTECODE_RA => Self::BytecodeRa(field(&mut reader)?),
            tag::RAM_RA => Self::RamRa(field(&mut reader)?),
            tag::TRUSTED_ADVICE => Self::TrustedAdvice,
            tag::UNTRUSTED_ADVICE => Self::UntrustedAdvice,
            tag::BYTECODE_CHUNK => Self::BytecodeChunk(field(&mut reader)?),
            tag::PROGRAM_IMAGE_INIT => Self::ProgramImageInit,
            tag::UNSIGNED_INC_CHUNK => Self::UnsignedIncChunk(field(&mut reader)?),
            tag::UNSIGNED_INC_MSB => Self::UnsignedIncMsb,
            tag::BYTECODE_REGISTER_SELECTOR => {
                let chunk = field(&mut reader)?;
                Self::BytecodeRegisterSelector(chunk, field(&mut reader)?)
            }
            tag::BYTECODE_CIRCUIT_FLAG => {
                let chunk = field(&mut reader)?;
                Self::BytecodeCircuitFlag(chunk, field(&mut reader)?)
            }
            tag::BYTECODE_INSTRUCTION_FLAG => {
                let chunk = field(&mut reader)?;
                Self::BytecodeInstructionFlag(chunk, field(&mut reader)?)
            }
            tag::BYTECODE_LOOKUP_SELECTOR => Self::BytecodeLookupSelector(field(&mut reader)?),
            tag::BYTECODE_RAF_FLAG => Self::BytecodeRafFlag(field(&mut reader)?),
            tag::BYTECODE_UNEXPANDED_PC_BYTES => {
                Self::BytecodeUnexpandedPcBytes(field(&mut reader)?)
            }
            tag::BYTECODE_IMM_BYTES => Self::BytecodeImmBytes(field(&mut reader)?),
            tag::PROGRAM_IMAGE_BYTES => Self::ProgramImageBytes,
            _ => return Err(SerializationError::InvalidData),
        })
    }
}

impl CanonicalSerialize for VirtualPolynomial {
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        match self {
            Self::PC => 0u8.serialize_with_mode(&mut writer, compress),
            Self::UnexpandedPC => 1u8.serialize_with_mode(&mut writer, compress),
            Self::NextPC => 2u8.serialize_with_mode(&mut writer, compress),
            Self::NextUnexpandedPC => 3u8.serialize_with_mode(&mut writer, compress),
            Self::NextIsNoop => 4u8.serialize_with_mode(&mut writer, compress),
            Self::NextIsVirtual => 5u8.serialize_with_mode(&mut writer, compress),
            Self::NextIsFirstInSequence => 6u8.serialize_with_mode(&mut writer, compress),
            Self::LeftLookupOperand => 7u8.serialize_with_mode(&mut writer, compress),
            Self::RightLookupOperand => 8u8.serialize_with_mode(&mut writer, compress),
            Self::LeftInstructionInput => 9u8.serialize_with_mode(&mut writer, compress),
            Self::RightInstructionInput => 10u8.serialize_with_mode(&mut writer, compress),
            Self::Product => 11u8.serialize_with_mode(&mut writer, compress),
            Self::ShouldJump => 12u8.serialize_with_mode(&mut writer, compress),
            Self::ShouldBranch => 13u8.serialize_with_mode(&mut writer, compress),
            Self::Rd => 14u8.serialize_with_mode(&mut writer, compress),
            Self::Imm => 15u8.serialize_with_mode(&mut writer, compress),
            Self::Rs1Value => 16u8.serialize_with_mode(&mut writer, compress),
            Self::Rs2Value => 17u8.serialize_with_mode(&mut writer, compress),
            Self::RdWriteValue => 18u8.serialize_with_mode(&mut writer, compress),
            Self::Rs1Ra => 19u8.serialize_with_mode(&mut writer, compress),
            Self::Rs2Ra => 20u8.serialize_with_mode(&mut writer, compress),
            Self::RdWa => 21u8.serialize_with_mode(&mut writer, compress),
            Self::LookupOutput => 22u8.serialize_with_mode(&mut writer, compress),
            Self::InstructionRaf => 23u8.serialize_with_mode(&mut writer, compress),
            Self::InstructionRafFlag => 24u8.serialize_with_mode(&mut writer, compress),
            Self::InstructionRa(i) => {
                25u8.serialize_with_mode(&mut writer, compress)?;
                u8::try_from(*i)
                    .map_err(|_| index_out_of_range())?
                    .serialize_with_mode(&mut writer, compress)
            }
            Self::RegistersVal => 26u8.serialize_with_mode(&mut writer, compress),
            Self::RamAddress => 27u8.serialize_with_mode(&mut writer, compress),
            Self::RamRa => 28u8.serialize_with_mode(&mut writer, compress),
            Self::RamReadValue => 29u8.serialize_with_mode(&mut writer, compress),
            Self::RamWriteValue => 30u8.serialize_with_mode(&mut writer, compress),
            Self::RamVal => 31u8.serialize_with_mode(&mut writer, compress),
            Self::RamValInit => 32u8.serialize_with_mode(&mut writer, compress),
            Self::RamValFinal => 33u8.serialize_with_mode(&mut writer, compress),
            Self::RamHammingWeight => 34u8.serialize_with_mode(&mut writer, compress),
            Self::UnivariateSkip => 35u8.serialize_with_mode(&mut writer, compress),
            Self::OpFlags(flags) => {
                36u8.serialize_with_mode(&mut writer, compress)?;
                u8::try_from(*flags as usize)
                    .map_err(|_| index_out_of_range())?
                    .serialize_with_mode(&mut writer, compress)
            }
            Self::InstructionFlags(flags) => {
                37u8.serialize_with_mode(&mut writer, compress)?;
                u8::try_from(*flags as usize)
                    .map_err(|_| index_out_of_range())?
                    .serialize_with_mode(&mut writer, compress)
            }
            Self::LookupTableFlag(flag) => {
                38u8.serialize_with_mode(&mut writer, compress)?;
                u8::try_from(*flag)
                    .map_err(|_| index_out_of_range())?
                    .serialize_with_mode(&mut writer, compress)
            }
            Self::BytecodeReadRafAddrClaim => 39u8.serialize_with_mode(&mut writer, compress),
            Self::BooleanityAddrClaim => 40u8.serialize_with_mode(&mut writer, compress),
            Self::BytecodeValClaim(i) => {
                41u8.serialize_with_mode(&mut writer, compress)?;
                u8::try_from(*i)
                    .map_err(|_| index_out_of_range())?
                    .serialize_with_mode(&mut writer, compress)
            }
            Self::BytecodeClaimReductionIntermediate => {
                42u8.serialize_with_mode(&mut writer, compress)
            }
            Self::ProgramImageInitContributionRw => 43u8.serialize_with_mode(&mut writer, compress),
            Self::FusedInc => 44u8.serialize_with_mode(&mut writer, compress),
        }
    }

    fn serialized_size(&self, _compress: Compress) -> usize {
        match self {
            Self::PC
            | Self::UnexpandedPC
            | Self::NextPC
            | Self::NextUnexpandedPC
            | Self::NextIsNoop
            | Self::NextIsVirtual
            | Self::NextIsFirstInSequence
            | Self::LeftLookupOperand
            | Self::RightLookupOperand
            | Self::LeftInstructionInput
            | Self::RightInstructionInput
            | Self::Product
            | Self::ShouldJump
            | Self::ShouldBranch
            | Self::Rd
            | Self::Imm
            | Self::Rs1Value
            | Self::Rs2Value
            | Self::RdWriteValue
            | Self::Rs1Ra
            | Self::Rs2Ra
            | Self::RdWa
            | Self::LookupOutput
            | Self::InstructionRaf
            | Self::InstructionRafFlag
            | Self::RegistersVal
            | Self::RamAddress
            | Self::RamRa
            | Self::RamReadValue
            | Self::RamWriteValue
            | Self::RamVal
            | Self::RamValInit
            | Self::RamValFinal
            | Self::RamHammingWeight
            | Self::UnivariateSkip
            | Self::BytecodeReadRafAddrClaim
            | Self::BooleanityAddrClaim
            | Self::BytecodeClaimReductionIntermediate
            | Self::ProgramImageInitContributionRw
            | Self::FusedInc => 1,
            Self::InstructionRa(_)
            | Self::OpFlags(_)
            | Self::InstructionFlags(_)
            | Self::LookupTableFlag(_)
            | Self::BytecodeValClaim(_) => 2,
        }
    }
}

impl Valid for VirtualPolynomial {
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

impl CanonicalDeserialize for VirtualPolynomial {
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        Ok(
            match u8::deserialize_with_mode(&mut reader, compress, validate)? {
                0 => Self::PC,
                1 => Self::UnexpandedPC,
                2 => Self::NextPC,
                3 => Self::NextUnexpandedPC,
                4 => Self::NextIsNoop,
                5 => Self::NextIsVirtual,
                6 => Self::NextIsFirstInSequence,
                7 => Self::LeftLookupOperand,
                8 => Self::RightLookupOperand,
                9 => Self::LeftInstructionInput,
                10 => Self::RightInstructionInput,
                11 => Self::Product,
                12 => Self::ShouldJump,
                13 => Self::ShouldBranch,
                14 => Self::Rd,
                15 => Self::Imm,
                16 => Self::Rs1Value,
                17 => Self::Rs2Value,
                18 => Self::RdWriteValue,
                19 => Self::Rs1Ra,
                20 => Self::Rs2Ra,
                21 => Self::RdWa,
                22 => Self::LookupOutput,
                23 => Self::InstructionRaf,
                24 => Self::InstructionRafFlag,
                25 => {
                    let i = u8::deserialize_with_mode(&mut reader, compress, validate)?;
                    Self::InstructionRa(i as usize)
                }
                26 => Self::RegistersVal,
                27 => Self::RamAddress,
                28 => Self::RamRa,
                29 => Self::RamReadValue,
                30 => Self::RamWriteValue,
                31 => Self::RamVal,
                32 => Self::RamValInit,
                33 => Self::RamValFinal,
                34 => Self::RamHammingWeight,
                35 => Self::UnivariateSkip,
                36 => {
                    let discriminant = u8::deserialize_with_mode(&mut reader, compress, validate)?;
                    let flags = CircuitFlags::from_repr(discriminant)
                        .ok_or(SerializationError::InvalidData)?;
                    Self::OpFlags(flags)
                }
                37 => {
                    let discriminant = u8::deserialize_with_mode(&mut reader, compress, validate)?;
                    let flags = InstructionFlags::from_repr(discriminant)
                        .ok_or(SerializationError::InvalidData)?;
                    Self::InstructionFlags(flags)
                }
                38 => {
                    let flag = u8::deserialize_with_mode(&mut reader, compress, validate)?;
                    Self::LookupTableFlag(flag as usize)
                }
                39 => Self::BytecodeReadRafAddrClaim,
                40 => Self::BooleanityAddrClaim,
                41 => {
                    let i = u8::deserialize_with_mode(&mut reader, compress, validate)?;
                    Self::BytecodeValClaim(i as usize)
                }
                42 => Self::BytecodeClaimReductionIntermediate,
                43 => Self::ProgramImageInitContributionRw,
                44 => Self::FusedInc,
                _ => return Err(SerializationError::InvalidData),
            },
        )
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn roundtrip<T>(value: &T)
    where
        T: CanonicalSerialize + CanonicalDeserialize + PartialEq + std::fmt::Debug,
    {
        let mut bytes = Vec::new();
        value
            .serialize_with_mode(&mut bytes, Compress::Yes)
            .unwrap();
        assert_eq!(
            bytes.len(),
            value.serialized_size(Compress::Yes),
            "serialized_size disagrees with the encoding for {value:?}"
        );
        let decoded = T::deserialize_with_mode(bytes.as_slice(), Compress::Yes, Validate::Yes)
            .unwrap_or_else(|error| panic!("decode failed for {value:?}: {error:?}"));
        assert_eq!(&decoded, value);
    }

    /// One instance of every variant. The exhaustive match makes a new
    /// `CommittedPolynomial` variant a compile error here, and the count
    /// assertion makes it a test failure until the list below covers it too.
    fn all_committed() -> Vec<CommittedPolynomial> {
        fn coverage_witness(polynomial: CommittedPolynomial) {
            match polynomial {
                CommittedPolynomial::RdInc
                | CommittedPolynomial::RamInc
                | CommittedPolynomial::InstructionRa(_)
                | CommittedPolynomial::BytecodeRa(_)
                | CommittedPolynomial::RamRa(_)
                | CommittedPolynomial::TrustedAdvice
                | CommittedPolynomial::UntrustedAdvice
                | CommittedPolynomial::BytecodeChunk(_)
                | CommittedPolynomial::ProgramImageInit
                | CommittedPolynomial::UnsignedIncChunk(_)
                | CommittedPolynomial::UnsignedIncMsb
                | CommittedPolynomial::BytecodeRegisterSelector(..)
                | CommittedPolynomial::BytecodeCircuitFlag(..)
                | CommittedPolynomial::BytecodeInstructionFlag(..)
                | CommittedPolynomial::BytecodeLookupSelector(_)
                | CommittedPolynomial::BytecodeRafFlag(_)
                | CommittedPolynomial::BytecodeUnexpandedPcBytes(_)
                | CommittedPolynomial::BytecodeImmBytes(_)
                | CommittedPolynomial::ProgramImageBytes => {}
            }
        }
        let all = vec![
            CommittedPolynomial::RdInc,
            CommittedPolynomial::RamInc,
            CommittedPolynomial::InstructionRa(3),
            CommittedPolynomial::BytecodeRa(2),
            CommittedPolynomial::RamRa(1),
            CommittedPolynomial::TrustedAdvice,
            CommittedPolynomial::UntrustedAdvice,
            CommittedPolynomial::BytecodeChunk(4),
            CommittedPolynomial::ProgramImageInit,
            CommittedPolynomial::UnsignedIncChunk(5),
            CommittedPolynomial::UnsignedIncMsb,
            CommittedPolynomial::BytecodeRegisterSelector(1, 2),
            CommittedPolynomial::BytecodeCircuitFlag(0, 7),
            CommittedPolynomial::BytecodeInstructionFlag(1, 3),
            CommittedPolynomial::BytecodeLookupSelector(0),
            CommittedPolynomial::BytecodeRafFlag(1),
            CommittedPolynomial::BytecodeUnexpandedPcBytes(0),
            CommittedPolynomial::BytecodeImmBytes(1),
            CommittedPolynomial::ProgramImageBytes,
        ];
        all.iter().copied().for_each(coverage_witness);
        assert_eq!(
            all.len(),
            <CommittedPolynomial as strum::EnumCount>::COUNT,
            "every CommittedPolynomial variant needs a round-trip representative"
        );
        all
    }

    /// One instance of every variant, same coverage pattern as
    /// [`all_committed`].
    fn all_virtual() -> Vec<VirtualPolynomial> {
        fn coverage_witness(polynomial: VirtualPolynomial) {
            match polynomial {
                VirtualPolynomial::PC
                | VirtualPolynomial::UnexpandedPC
                | VirtualPolynomial::NextPC
                | VirtualPolynomial::NextUnexpandedPC
                | VirtualPolynomial::NextIsNoop
                | VirtualPolynomial::NextIsVirtual
                | VirtualPolynomial::NextIsFirstInSequence
                | VirtualPolynomial::LeftLookupOperand
                | VirtualPolynomial::RightLookupOperand
                | VirtualPolynomial::LeftInstructionInput
                | VirtualPolynomial::RightInstructionInput
                | VirtualPolynomial::Product
                | VirtualPolynomial::ShouldJump
                | VirtualPolynomial::ShouldBranch
                | VirtualPolynomial::Rd
                | VirtualPolynomial::Imm
                | VirtualPolynomial::Rs1Value
                | VirtualPolynomial::Rs2Value
                | VirtualPolynomial::RdWriteValue
                | VirtualPolynomial::Rs1Ra
                | VirtualPolynomial::Rs2Ra
                | VirtualPolynomial::RdWa
                | VirtualPolynomial::LookupOutput
                | VirtualPolynomial::InstructionRaf
                | VirtualPolynomial::InstructionRafFlag
                | VirtualPolynomial::InstructionRa(_)
                | VirtualPolynomial::RegistersVal
                | VirtualPolynomial::RamAddress
                | VirtualPolynomial::RamRa
                | VirtualPolynomial::RamReadValue
                | VirtualPolynomial::RamWriteValue
                | VirtualPolynomial::RamVal
                | VirtualPolynomial::RamValInit
                | VirtualPolynomial::RamValFinal
                | VirtualPolynomial::RamHammingWeight
                | VirtualPolynomial::UnivariateSkip
                | VirtualPolynomial::OpFlags(_)
                | VirtualPolynomial::InstructionFlags(_)
                | VirtualPolynomial::LookupTableFlag(_)
                | VirtualPolynomial::BytecodeReadRafAddrClaim
                | VirtualPolynomial::BooleanityAddrClaim
                | VirtualPolynomial::BytecodeValClaim(_)
                | VirtualPolynomial::BytecodeClaimReductionIntermediate
                | VirtualPolynomial::ProgramImageInitContributionRw
                | VirtualPolynomial::FusedInc => {}
            }
        }
        let all = vec![
            VirtualPolynomial::PC,
            VirtualPolynomial::UnexpandedPC,
            VirtualPolynomial::NextPC,
            VirtualPolynomial::NextUnexpandedPC,
            VirtualPolynomial::NextIsNoop,
            VirtualPolynomial::NextIsVirtual,
            VirtualPolynomial::NextIsFirstInSequence,
            VirtualPolynomial::LeftLookupOperand,
            VirtualPolynomial::RightLookupOperand,
            VirtualPolynomial::LeftInstructionInput,
            VirtualPolynomial::RightInstructionInput,
            VirtualPolynomial::Product,
            VirtualPolynomial::ShouldJump,
            VirtualPolynomial::ShouldBranch,
            VirtualPolynomial::Rd,
            VirtualPolynomial::Imm,
            VirtualPolynomial::Rs1Value,
            VirtualPolynomial::Rs2Value,
            VirtualPolynomial::RdWriteValue,
            VirtualPolynomial::Rs1Ra,
            VirtualPolynomial::Rs2Ra,
            VirtualPolynomial::RdWa,
            VirtualPolynomial::LookupOutput,
            VirtualPolynomial::InstructionRaf,
            VirtualPolynomial::InstructionRafFlag,
            VirtualPolynomial::InstructionRa(6),
            VirtualPolynomial::RegistersVal,
            VirtualPolynomial::RamAddress,
            VirtualPolynomial::RamRa,
            VirtualPolynomial::RamReadValue,
            VirtualPolynomial::RamWriteValue,
            VirtualPolynomial::RamVal,
            VirtualPolynomial::RamValInit,
            VirtualPolynomial::RamValFinal,
            VirtualPolynomial::RamHammingWeight,
            VirtualPolynomial::UnivariateSkip,
            VirtualPolynomial::OpFlags(CircuitFlags::Store),
            VirtualPolynomial::InstructionFlags(InstructionFlags::from_repr(0).unwrap()),
            VirtualPolynomial::LookupTableFlag(7),
            VirtualPolynomial::BytecodeReadRafAddrClaim,
            VirtualPolynomial::BooleanityAddrClaim,
            VirtualPolynomial::BytecodeValClaim(1),
            VirtualPolynomial::BytecodeClaimReductionIntermediate,
            VirtualPolynomial::ProgramImageInitContributionRw,
            VirtualPolynomial::FusedInc,
        ];
        all.iter().copied().for_each(coverage_witness);
        assert_eq!(
            all.len(),
            <VirtualPolynomial as strum::EnumCount>::COUNT,
            "every VirtualPolynomial variant needs a round-trip representative"
        );
        all
    }

    fn all_sumchecks() -> impl Iterator<Item = SumcheckId> {
        (0..SumcheckId::COUNT).map(|i| SumcheckId::from_u8(u8::try_from(i).unwrap()).unwrap())
    }

    #[test]
    fn committed_polynomial_codec_roundtrips_every_variant() {
        for polynomial in all_committed() {
            roundtrip(&polynomial);
        }
    }

    #[test]
    fn virtual_polynomial_codec_roundtrips_every_variant() {
        for polynomial in all_virtual() {
            roundtrip(&polynomial);
        }
    }

    #[test]
    fn opening_id_codec_roundtrips_every_variant_and_sumcheck() {
        for sumcheck in all_sumchecks() {
            roundtrip(&OpeningId::UntrustedAdvice(sumcheck));
            roundtrip(&OpeningId::TrustedAdvice(sumcheck));
            for polynomial in all_committed() {
                roundtrip(&OpeningId::committed(polynomial, sumcheck));
            }
            for polynomial in all_virtual() {
                roundtrip(&OpeningId::virt(polynomial, sumcheck));
            }
        }
    }
}
