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

/// Clears the entire BSS segment of the kernel image. This may corrupt kernel memory if the
/// function is executed after data in the BSS segment has been mutated. Furthermore, this function
/// assumes that the symbols `__bss_start` and `_bss_end` defined in the linker script are valid and
/// well-aligned addresses.
#[no_mangle]
unsafe extern "C" fn clear_bss() {
    use core::{ops::Range, slice};

    unsafe {
        slice::from_mut_ptr_range::<u8>(Range {
            start: (&__bss_start as *const c_void).cast_mut().cast(),
            end: (&__bss_end as *const c_void).cast_mut().cast(),
        })
        .fill(0);
    }
}
