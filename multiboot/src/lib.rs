//! This crate contains the structures used to implement the multiboot boot protocol as defined in
//! the corresponding specification:
//!
//! https://www.gnu.org/software/grub/manual/multiboot/multiboot.html (version 0.6.96)

#![no_std]

mod header;
mod mmap;
mod module;

use module::Module;

pub use header::{Header, HeaderBuilder};

/// When the bootloader (e.g. GRUB) transfers control to the kernel, an instance of this struct is
/// passed along to the kernel. It contains information vital to the kernel startup procedure.
#[repr(C)]
#[derive(Debug)]
pub struct BootInfo {
    /// Indicates the presence and validity of other fields in the Multiboot information structure.
    /// Any set bits that the operating system does not understand should be ignored.
    flags: u32,

    /// If bit 0 in the `flags` word is set, then the `mem_lower` field is valid. `mem_lower`
    /// indicates the amount of lower memory available in kilobytes. Lower memory starts at
    /// address 0 The maximum possible value for lower memory is 640 kilobytes.
    mem_lower: u32,

    /// If bit 0 in the `flags` word is set, then the `mem_upper` field is valid. `mem_upper`
    /// indicates the amount of upper memory in kilobytes. Upper memory starts at address 1
    /// megabyte. The value returned for upper memory is maximally the address of the first
    /// upper memory hole minus 1 megabyte. It is not guaranteed to be this value.
    mem_upper: u32,

    /// If bit 1 in the `flags` word is set, then the `boot_device` field is valid, and indicates
    /// which BIOS disk device the boot loader loaded the OS image from. The operating system may
    /// use this field as a hint for determining its own root device, but is not required to.
    _boot_device: u32,

    /// If bit 2 of the `flags` word is set, the `cmdline` field is valid, and contains the
    /// physical address of the command line to be passed to the kernel. The command line is a
    /// normal C-style zero-terminated string.
    cmdline: u32,

    /// If bit 3 of `flags` is set, then the `mods` fields indicate to the kernel what boot
    /// modules were loaded along with the kernel image, and where they can be found. `mods_count`
    /// contains the number of modules loaded, it may be zero, indicating no boot modules were
    /// loaded, even if bit 3 of `flags` is set.
    mods_count: u32,

    /// If bit 3 of `flags` is set, then the `mods` fields indicate to the kernel what boot modules
    /// were loaded along with the kernel image, and where they can be found. `mods_addr` contains
    /// the physical address of the first module structure. For details each module's structure see
    /// the [Module] structure.
    mods_addr: u32,

    _unused: [u32; 4],

    /// If bit 6 in the `flags` word is set, then the `mmap_length` field is valid and indicates
    /// the address and length of a buffer containing a memory map of the machine provided by the
    /// BIOS.
    mmap_length: u32,

    /// If bit 6 in the `flags` word is set, then the `mmap_length` field is valid and indicates
    /// the start of a buffer containing a memory map of the machine provided by the BIOS.
    /// `mmap_length` contains the total size of the buffer. The buffer consists of one or more
    /// memory map entries. For details on their layout see [MemoryMapEntry] documentation. The
    /// map provided is guaranteed to list all standard RAM that should be available for normal
    /// use.
    mmap: u32,
}

impl BootInfo {
    /// ### Safety
    ///
    /// * Memory pointed to by multiboot pointer must not be mutated for the lifetime `'mb`.
    pub unsafe fn from_ptr<'mb>(magic: u32, mb_ptr: *const Self) -> &'mb BootInfo {
        // Check multiboot magic value and try to dereference pointer to information structure
        assert_eq!(magic, 0x2badb002, "Multiboot magic value mismatch");
        assert!(mb_ptr.is_aligned(), "Multiboot pointer must be aligned");
        unsafe {
            // Safety: Checked for alignment
            mb_ptr
                .as_ref()
                .expect("Multiboot information structure pointer should be non-null")
        }
    }
}

trait BitfieldExt {
    /// Check whether a specific bit is set
    fn is_nth_bit_set(&self, bit: usize) -> bool;
}

impl BitfieldExt for u32 {
    fn is_nth_bit_set(&self, bit: usize) -> bool {
        *self & (bit << 1) as u32 != 0
    }
}

impl BootInfo {
    /// Returns the kernel command line if one has been passed along by the bootloader.
    pub fn command_line(&self) -> Option<&core::ffi::CStr> {
        let cmdline_ptr = self.cmdline as *const core::ffi::c_char;
        if self.flags.is_nth_bit_set(2) && !cmdline_ptr.is_null() {
            Some(unsafe { core::ffi::CStr::from_ptr(cmdline_ptr) })
        } else {
            None
        }
    }

    /// If present, returns a slice of modules passed on to the kernel by the bootloader.
    pub fn modules(&self) -> Option<&[Module]> {
        let mods_ptr = self.mods_addr as *const Module;
        if self.flags.is_nth_bit_set(3) & !mods_ptr.is_null() {
            Some(unsafe { core::slice::from_raw_parts(mods_ptr, self.mods_count as usize) })
        } else {
            None
        }
    }

    /// This function returns an iterator that can be used to traverse the memory map passed on to
    /// the kernel by the bootloader or `None` if there is no memory map present.
    pub fn memory_map<'mb>(&'mb self) -> Option<mmap::MemoryMapIter<'mb>> {
        use core::slice;

        let mmap_ptr = self.mmap as *const u8;
        if self.flags.is_nth_bit_set(6) && !mmap_ptr.is_null() {
            let mmap_buffer = unsafe {
                // SAFETY: We just checked that the memory map is present and the pointer to its
                // memory is non-null. Also, we explicitly make sure that the lifetime of the
                // resulting reference is tied to the lifetime of the BootInfo struct.
                slice::from_raw_parts::<'mb>(mmap_ptr, self.mmap_length as usize)
            };
            Some(mmap_buffer.into())
        } else {
            None
        }
    }
}
