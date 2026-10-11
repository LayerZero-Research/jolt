//! Stop requests (SIGINT/SIGTERM) delivered to the runner.

use libc::{SIGINT, SIGTERM};
use std::sync::atomic::{AtomicU32, Ordering};

static SIGNALS: AtomicU32 = AtomicU32::new(0);

extern "C" fn on_signal(_: libc::c_int) {
    SIGNALS.fetch_add(1, Ordering::SeqCst);
}

pub fn install() {
    // SAFETY: the handler only touches an atomic.
    unsafe {
        libc::signal(SIGINT, on_signal as *const () as usize);
        libc::signal(SIGTERM, on_signal as *const () as usize);
    }
}

/// Stop signals received so far; a second one forces shutdown.
pub fn received() -> u32 {
    SIGNALS.load(Ordering::SeqCst)
}
