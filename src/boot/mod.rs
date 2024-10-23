//! ## Boot procedure implementation
//!
//! A dedicated bootloader (e.g. GRUB) must be used to load the kernel image into memory and pass
//! control to it. The bootloader must also provide the kernel with information about the machine
//! and its configuration (e.g. memory map, command line arguments, etc.). The modalities of these
//! tasks are defined by the boot protocol.

#[cfg(target_arch = "x86")]
mod x86;

/// Clears the entire BSS segment of the kernel image. This may corrupt kernel memory if the
/// function is executed after data in the BSS segment has been mutated. Furthermore, this function
/// assumes that the symbols `__bss_start` and `_bss_end` defined in the linker script are valid and
/// well-aligned addresses.
#[no_mangle]
unsafe extern "C" fn clear_bss() {
    use core::{ops::Range, slice};

    // Symbols defined by linker script:
    extern "C" {
        /// Start address of the BSS segment.
        static __bss_start: u8;

        /// End address of the BSS segment.
        static __bss_end: u8;
    }

    unsafe {
        slice::from_mut_ptr_range(Range {
            start: (&__bss_start as *const u8).cast_mut(),
            end: (&__bss_end as *const u8).cast_mut(),
        })
        .fill(0);
    }
}
