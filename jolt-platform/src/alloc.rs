use core::alloc::{GlobalAlloc, Layout};

pub struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        sys_alloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

extern "C" {
    static _HEAP_PTR: u8;
}

static mut ALLOC_NEXT: usize = 0;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn sys_alloc(size: usize, align: usize) -> *mut u8 {
    let mut next = unsafe { ALLOC_NEXT };

    if next == 0 {
        next = unsafe { (&_HEAP_PTR) as *const u8 as usize };
    }

    next = align_up(next, align);

    let ptr = next as *mut u8;
    next += size;

    unsafe { ALLOC_NEXT = next };
    ptr
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn sys_panic(msg_ptr: *const u8, len: usize) -> ! {
    // For now, we'll implement a simple panic that loops forever
    // In a more complete implementation, this could trigger a specific
    // ecall to notify the host about the panic
    
    // // Try to print the panic message if possible
    // if !msg_ptr.is_null() && len > 0 {
    //     let slice = unsafe { core::slice::from_raw_parts(msg_ptr, len) };
    //     if let Ok(msg) = core::str::from_utf8(slice) {
    //         // Use the print functionality if available
    //         #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    //         {
    //             crate::print::print("PANIC: ");
    //             crate::print::println(msg);
    //         }
    //         #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
    //         let _ = msg; // Avoid unused variable warning
    //     }
    // }
    
    // Infinite loop - in a real implementation this might be a specific ecall
    loop {
        #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
        unsafe {
            core::arch::asm!("nop", options(nomem, nostack, preserves_flags));
        }
        #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
        core::hint::spin_loop();
    }
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn sys_write(fd: u32, write_buf: *const u8, nbytes: usize) {
    // For a basic implementation, we'll only handle stdout/stderr (fd 1 and 2)
    // and ignore other file descriptors
    
    // if (fd == 1 || fd == 2) && !write_buf.is_null() && nbytes > 0 {
    //     let slice = unsafe { core::slice::from_raw_parts(write_buf, nbytes) };
    //     if let Ok(text) = core::str::from_utf8(slice) {
    //         #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    //         crate::print::print(text);
    //         #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
    //         let _ = text; // Avoid unused variable warning
    //     }
    // }
    // For other file descriptors or invalid parameters, we do nothing
    // In a more complete implementation, this could return an error code
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn sys_alloc_aligned(nwords: usize, align: usize) -> *mut u8 {
    // Use the existing sys_alloc function, converting words to bytes
    // Assuming word size is pointer width (8 bytes on 64-bit, 4 bytes on 32-bit)
    let word_size = core::mem::size_of::<usize>();
    let nbytes = nwords * word_size;
    
    sys_alloc(nbytes, align)
}
