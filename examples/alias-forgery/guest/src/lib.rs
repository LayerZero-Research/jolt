//! Target guest for the fp128 lookup-address alias soundness PoC.
//!
//! See `AKITA_FP128_LOOKUP_ADDRESS_SOUNDNESS.md`. The whole program is one
//! 64-bit register-register `ADD` whose result is the public output, so a
//! forged lookup address for that single cycle is directly observable in what
//! the verifier accepts.
//!
//! Two properties make it a usable PoC target:
//!
//! 1. The operand pair the PoC forges is passed in as input and occurs exactly
//!    once in the trace, so the forgery touches one cycle and nothing else.
//!    (`fp128_alias_forgery_e2e` asserts the gate fired exactly once.)
//! 2. Nothing in the guest re-derives or checks `total`. A guest assertion
//!    would not help anyway — the forged trace feeds the corrupted value back
//!    into the emulator, so the guest is never shown a contradiction. Only the
//!    verifier is in a position to reject, and it does not.

#![cfg_attr(feature = "guest", no_std)]

#[jolt::provable(heap_size = 32768, max_trace_length = 65536)]
fn settle(debit: u64, credit: u64) -> u64 {
    debit.wrapping_add(credit)
}
