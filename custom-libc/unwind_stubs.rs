#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Remove rust_eh_personality since std provides it
// #[no_mangle]
// pub extern "C" fn rust_eh_personality() {}

#[no_mangle]
pub extern "C" fn rust_begin_unwind(_info: *const u8) -> ! {
    // In ZKVM, we can't unwind, so we just abort
    loop {}
}

#[no_mangle]  
pub extern "C" fn _Unwind_Resume(_exception: *mut u8) {
    // No-op for bare metal
}

#[no_mangle]
pub extern "C" fn _Unwind_DeleteException(_exception: *mut u8) {
    // No-op for bare metal  
}

#[no_mangle]
pub extern "C" fn _Unwind_GetLanguageSpecificData(_context: *mut u8) -> *mut u8 {
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn _Unwind_GetRegionStart(_context: *mut u8) -> usize {
    0
}

#[no_mangle]
pub extern "C" fn _Unwind_GetTextRelBase(_context: *mut u8) -> usize {
    0
}

#[no_mangle]
pub extern "C" fn _Unwind_GetDataRelBase(_context: *mut u8) -> usize {
    0
}