/// The multiboot header must be present in the first 8KB of every multiboot-compliant kernel image.
/// It is used to indicate to the bootloader which features and information the kernel requires.
#[derive(Debug, Clone, PartialEq)]
#[repr(C, packed)]
pub struct Header {
    magic: u32,
    flags: u32,
    checksum: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
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
