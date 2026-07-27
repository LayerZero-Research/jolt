#![cfg_attr(feature = "guest", no_std)]

#[jolt::provable(heap_size = 32768, max_trace_length = 65536)]
fn untrusted_advice_consumer(public_value: u64, untrusted: jolt::UntrustedAdvice<u64>) -> u64 {
    assert_eq!(*untrusted, public_value);
    (*untrusted).wrapping_mul(3)
}
