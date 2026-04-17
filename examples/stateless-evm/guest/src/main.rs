#![no_main]

#[cfg(target_arch = "riscv64")]
mod guest_critical_section {
    use critical_section::{set_impl, Impl, RawRestoreState};

    struct SingleThreadedCriticalSection;
    set_impl!(SingleThreadedCriticalSection);

    unsafe impl Impl for SingleThreadedCriticalSection {
        unsafe fn acquire() -> RawRestoreState {}

        unsafe fn release(_: RawRestoreState) {}
    }
}

#[expect(unused_imports, reason = "re-exports guest-exposed items")]
use stateless_evm_guest::*;
