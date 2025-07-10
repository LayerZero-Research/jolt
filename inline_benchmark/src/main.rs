#![no_std]
#![no_main]

use core::{hint::black_box, panic::PanicInfo};

#[cfg(feature = "sha2_hasher")]
use sha2::{Digest, Sha256};

#[cfg(feature = "inline_sha2_hasher")]
use inline_sha2::sha256::Sha256;

#[cfg(feature = "sha3_hasher")]
use sha3::{Digest, Sha3_256};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let data: &'static [u8; 11] = b"hello world";

    #[cfg(feature = "sha2_hasher")]
    let mut hasher = Sha256::new();

    #[cfg(feature = "inline_sha2_hasher")]
    let mut hasher = Sha256::new();

    #[cfg(feature = "sha3_hasher")]
    let mut hasher = Sha3_256::new();

    hasher.update(data);
    let result: [u8; 32] = hasher.finalize().into();
    black_box(result);
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
