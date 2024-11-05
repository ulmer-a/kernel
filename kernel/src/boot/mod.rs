//! ## Boot procedure implementation
//!
//! A dedicated bootloader (e.g. GRUB) must be used to load the kernel image into memory and pass
//! control to it. The bootloader must also provide the kernel with information about the machine
//! and its configuration (e.g. memory map, command line arguments, etc.). The modalities of these
//! tasks are defined by the boot protocol.

use core::ffi::c_void;

#[cfg(target_arch = "x86")]
mod x86;

// Symbols defined by linker script:
extern "C" {
    /// Code start address
    static __text_start: c_void;

    /// Data end address
    static __data_end: c_void;

    /// Start address of the BSS segment.
    static __bss_start: c_void;

    /// End address of the BSS segment.
    static __bss_end: c_void;
}

/// Clear the BSS segment of the kernel image assuming it is is located at the addresses defined by
/// the `__bss_start` and `__bss_end` symbols defined in the linker script.
///
/// ## Safety
///
/// This must be executed before any other rust code starts, and must not be executed after that.
#[no_mangle]
unsafe extern "C" fn clear_bss() {
    use core::ptr;
    use core::{ops::Range, slice};

    extern "C" {
        /// Start address of the BSS segment.
        static mut __bss_start: u8;

        /// End address of the BSS segment.
        static mut __bss_end: u8;
    }

    unsafe {
        slice::from_mut_ptr_range::<u8>(Range {
            start: ptr::addr_of_mut!(__bss_start),
            end: ptr::addr_of_mut!(__bss_end),
        })
        .fill(0);
    }
}
