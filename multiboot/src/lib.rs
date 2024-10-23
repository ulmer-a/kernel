#![cfg_attr(not(test), no_std)]

//! This module contains the structures used to implement the multiboot boot protocol as defined in
//! the corresponding specification:
//!
//! https://www.gnu.org/software/grub/manual/multiboot/multiboot.html (version 0.6.96)

// Multiboot is only specified for `x86` (IA-32) architecture
#![cfg(target_arch = "x86")]

use types::mem::{MemoryChunk, MemoryChunkClass};

/// The multiboot header must be present in the first 8KB of every multiboot-compliant kernel image.
/// It is used to indicate to the bootloader which features and information the kernel requires.
#[repr(C, packed)]
pub struct Header {
    magic: u32,
    flags: u32,
    checksum: u32,
}

#[derive(Debug, Default)]
pub struct HeaderBuilder {
    flags: u32,
}

/// Construct a basic valid multiboot header with none of the request flags enabled. Use the
/// builder methods to enable specific features.
impl HeaderBuilder {
    /// Create a new header builder with default flags. This could be replaced with
    /// Default::default() if it was const. But it isn't so plase use new() if you want a const
    /// object.
    pub const fn new() -> Self {
        Self { flags: 0 }
    }

    /// Requests that any modules loaded by the bootloader be aligned on page boundaries (4K).
    pub const fn request_aligned_modules(self) -> Self {
        Self {
            flags: self.flags | 1,
        }
    }

    /// Requests a memory map from the bootloader by setting the corresponding header flag.
    pub const fn request_memory_map(self) -> Self {
        Self {
            flags: self.flags | 2,
        }
    }

    /// Build a valid multiboot header using the selected flags and compute the header checksum.
    pub const fn build(self) -> Header {
        const HEADER_MAGIC: u32 = 0x1bad_b002;
        // The `magic`, `flags` and `checksum` fields must have an unsigned sum of zero.
        Header {
            magic: HEADER_MAGIC,
            flags: self.flags,
            checksum: !HEADER_MAGIC.wrapping_add(self.flags) + 1,
        }
    }
}

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

pub trait BitfieldExt {
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
    pub fn _command_line(&self) -> Option<&core::ffi::CStr> {
        if self.flags.is_nth_bit_set(2) && !self.cmdline.is_null() {
            Some(unsafe { core::ffi::CStr::from_ptr(self.cmdline) })
        } else {
            None
        }
    }

    /// If present, returns a slice of modules passed on to the kernel by the bootloader.
    pub fn _modules(&self) -> Option<&[_Module]> {
        if self.flags.is_nth_bit_set(3) & !self.mods_addr.is_null() {
            Some(unsafe { core::slice::from_raw_parts(self.mods_addr, self.mods_count) })
        } else {
            None
        }
    }

    /// This function returns an iterator that can be used to traverse the memory map passed on to
    /// the kernel by the bootloader or `None` if there is no memory map present.
    pub fn memory_map<'mb>(&'mb self) -> Option<impl Iterator<Item = MemoryChunk> + Clone + 'mb> {
        use core::slice;

        if self.flags.is_nth_bit_set(6) && !self.mmap.is_null() {
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
