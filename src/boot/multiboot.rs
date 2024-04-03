//! This module contains the structures used to implement the multiboot boot protocol as defined in
//! the corresponding specification:
//!
//! https://www.gnu.org/software/grub/manual/multiboot/multiboot.html (version 0.6.96)

// Multiboot is only specified for `x86` (IA-32) architecture
#![cfg(target_arch = "x86")]

use crate::mem::physical::{MemoryChunk, MemoryChunkClass};

/// The multiboot header must be present in the first 8KB of every multiboot-compliant kernel image.
/// It is used to indicate to the bootloader which features and information the kernel requires.
#[repr(C, packed)]
pub struct Header {
    magic: u32,
    flags: u32,
    checksum: u32,
}

impl Header {
    /// Construct a basic valid multiboot header with none of the request flags enabled. Use the
    /// builder methods to enable specific features.
    pub const fn new() -> Self {
        Self {
            magic: 0x1bad_b002,
            flags: 0,
            checksum: 0,
        }
        .with_checksum()
    }

    /// Requests that any modules loaded by the bootloader be aligned on page boundaries (4K).
    pub const fn request_aligned_modules(self) -> Self {
        Self {
            flags: self.flags | 1,
            ..self
        }
        .with_checksum()
    }

    /// Requests a memory map from the bootloader by setting the corresponding header flag.
    pub const fn request_memory_map(self) -> Self {
        Self {
            flags: self.flags | 2,
            ..self
        }
        .with_checksum()
    }

    /// Computes the header checksum which needs to be correct in order to form a valid multiboot
    /// header structure recognized by bootloaders. The `magic` and `flags` and `checksum` fields
    /// must have an unsigned sum of zero.
    const fn with_checksum(self) -> Self {
        Self {
            checksum: !(self.magic + self.flags) + 1,
            ..self
        }
    }
}

/// When the bootloader (e.g. GRUB) transfers control to the kernel, an instance of this struct is
/// passed along to the kernel. It contains information vital to the kernel startup procedure.
#[repr(C, packed)]
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
    cmdline: *const core::ffi::c_char,

    /// If bit 3 of `flags` is set, then the `mods` fields indicate to the kernel what boot
    /// modules were loaded along with the kernel image, and where they can be found. `mods_count`
    /// contains the number of modules loaded, it may be zero, indicating no boot modules were
    /// loaded, even if bit 3 of `flags` is set.
    mods_count: usize,

    /// If bit 3 of `flags` is set, then the `mods` fields indicate to the kernel what boot modules
    /// were loaded along with the kernel image, and where they can be found. `mods_addr` contains
    /// the physical address of the first module structure. For details each module's structure see
    /// the [Module] structure.
    mods_addr: *const _Module,

    _unused: [u32; 4],

    /// If bit 6 in the `flags` word is set, then the `mmap_length` field is valid and indicates
    /// the address and length of a buffer containing a memory map of the machine provided by the
    /// BIOS.
    mmap_length: usize,

    /// If bit 6 in the `flags` word is set, then the `mmap_length` field is valid and indicates
    /// the start of a buffer containing a memory map of the machine provided by the BIOS.
    /// `mmap_length` contains the total size of the buffer. The buffer consists of one or more
    /// memory map entries. For details on their layout see [MemoryMapEntry] documentation. The
    /// map provided is guaranteed to list all standard RAM that should be available for normal
    /// use.
    mmap: *const u8,
    // ...
}

impl BootInfo {
    /// Returns the kernel command line if one has been passed along by the bootloader.
    pub fn _command_line(&self) -> Option<&core::ffi::CStr> {
        const COMMAND_LINE_PRESENT: u32 = 1 << 2;
        if self.flags & COMMAND_LINE_PRESENT != 0 && !self.cmdline.is_null() {
            Some(unsafe { core::ffi::CStr::from_ptr(self.cmdline) })
        } else {
            None
        }
    }

    /// If present, returns a slice of modules passed on to the kernel by the bootloader.
    pub fn _modules(&self) -> Option<&[_Module]> {
        const MODULES_PRESENT: u32 = 1 << 3;
        if self.flags & MODULES_PRESENT != 0 && !self.mods_addr.is_null() {
            Some(unsafe { core::slice::from_raw_parts(self.mods_addr, self.mods_count) })
        } else {
            None
        }
    }

    /// This function returns an iterator that can be used to traverse the memory map passed on to
    /// the kernel by the bootloader or `None` if there is no memory map present.
    pub fn memory_map<'mb>(&'mb self) -> Option<impl Iterator<Item = MemoryChunk> + Clone + 'mb> {
        use core::slice;

        const MEMORY_MAP_PRESENT: u32 = 1 << 6;
        if self.flags & MEMORY_MAP_PRESENT != 0 && !self.mmap.is_null() {
            Some(MemoryMap {
                // SAFETY: We just checked that the memory map is present and the pointer to its
                // memory is non-null. Also, we explicitly make sure that the lifetime of the
                // resulting reference is tied to the lifetime of the BootInfo struct.
                buffer: unsafe { slice::from_raw_parts::<'mb>(self.mmap, self.mmap_length) },
            })
        } else {
            None
        }
    }
}

/// An entry in the bootloader-provided module list.
#[repr(C)]
pub struct _Module {
    /// Start address of the module.
    mod_start: u32,

    /// End address of the module.
    mod_end: u32,

    /// The `string` field provides an arbitrary zero-terminated ASCII string to be associated with
    /// that particular module. It may also be null if there is no associated string.
    string: *const core::ffi::c_char,

    /// Must be ignored by the OS.
    _reserved: u32,
}

/// Provides an iterator over the multiboot memory map.
#[derive(Clone)]
struct MemoryMap<'mb> {
    /// Reference to the buffer from [`BootInfo`] that contains all the memory map entries.
    buffer: &'mb [u8],
}

impl Iterator for MemoryMap<'_> {
    type Item = MemoryChunk;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: We want to take a reference to the first memory map entry that is contained
        // in the buffer. So, if the buffer is large enough, transmute the first matching bytes
        // to a MemoryMapEntry. This type has the correct layout and uses repr(C), so we should
        // be fine.
        let (head, body, _) = unsafe { self.buffer.align_to::<MemoryMapEntry>() };
        assert_eq!(head.len(), 0);

        let entry = body.first()?;
        self.buffer = &self.buffer[entry.offset_to_next()..];
        Some(entry.into())
    }
}

/// Represents an entry in the multiboot memory map. The buffer consists of one or more of the
/// following size/structure pairs (`size` is really just used for skipping to the next pair):
///
/// ```text
///         +-------------------+  <-- Start of struct MemoryMapEntry
/// -4      | size              |
///         +-------------------+  <-- `size` is from here to next entry
/// 0       | base_addr         |
/// 8       | length            |
/// 16      | type              |
///         +-------------------+  <-- End of struct MemoryMapEntry
/// ```
///
/// where `size` is the size of the associated structure in bytes, which can be greater than the
/// minimum of 20 bytes. `base_addr` is the starting address. `length` is the size of the memory
/// region in bytes.
#[repr(C)]
pub struct MemoryMapEntry {
    /// When 4 is added to `size`, the result can be used as an offset to skip to the next memory
    /// map entry in the mmap buffer. According to the specification, this offset can be larger
    /// than than the size of this structure (20 bytes).
    size: u32,

    /// The start address of the memory region described by this entry.
    base_addr: u64,

    /// The size in bytes of the memory region described by this entry.
    length: u64,

    /// The type of the memory described by this entry.
    ///
    /// * 1: "available RAM"
    /// * 3: "usable memory holding ACPI information"
    /// * 4: "reserved memory which needs to be preserved on hibernation"
    /// * 5: "memory which is occupied by defective RAM modules"
    r#type: u32,
}

impl MemoryMapEntry {
    /// Returns the offset from the start address of this memory map entry to the next entry in the
    /// buffer.
    pub fn offset_to_next(&self) -> usize {
        self.size as usize + 4
    }
}

// Silencing `from_over_into` here because the multiboot MemoryMapEntry struct is more specific than
// the generic MemoryChunk struct.
#[allow(clippy::from_over_into)]
impl Into<MemoryChunk> for &MemoryMapEntry {
    fn into(self) -> MemoryChunk {
        MemoryChunk {
            base_addr: self.base_addr,
            length: self.length,
            class: match self.r#type {
                1 => MemoryChunkClass::Available,
                _ => MemoryChunkClass::Unusable,
            },
        }
    }
}
