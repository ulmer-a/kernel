//! Every multiboot-compliant kernel needs to have the multiboot header structure within the first
//! 8K of its binary. It is recommended to link it into a custom section of the binary (e.g.
//! `.multiboot`) to make sure that it will actually end up in the first 8K.
//!
//! Example:
//!
//! ```
//! #[used]
//! #[link_section = ".multiboot"]
//! static MULTIBOOT_HEADER: Header = HeaderBuilder::new()
//!     .request_aligned_modules()
//!     .request_memory_map()
//!     .request_default_framebuffer()
//!     .build();
//! ```

/// The multiboot header must be present in the first 8KB of every multiboot-compliant kernel image.
/// It is used to indicate to the bootloader which features and information the kernel requires.
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct Header {
    /// The magic number identifying the header, which must be the hexadecimal value 0x1BADB002.
    magic: u32,

    /// The field specifies features that the OS image requests or requires of an boot loader. Bits
    /// 0-15 indicate requirements; if the boot loader sees any of these bits set but doesn’t
    /// understand the flag or can’t fulfill the requirements it indicates for some reason, it must
    /// notify the user and fail to load the OS image. Bits 16-31 indicate optional features; if
    /// any bits in this range are set but the boot loader doesn’t understand them, it may simply
    /// ignore them and proceed as usual. Naturally, all as-yet-undefined bits in the `flags` word
    /// must be set to zero in OS images. This way, the ‘flags’ fields serves for version control
    /// as well as simple feature selection.
    flags: u32,

    /// The field `checksum` is a 32-bit unsigned value which, when added to the other magic fields
    /// (i.e. `magic` and `flags`), must have a 32-bit unsigned sum of zero.
    checksum: u32,

    /// The load address request fields enabled by flag bit 16 are physical addresses.
    addresses: LoadAddressRequest,

    /// All of the graphics fields are enabled by flag bit 2. They specify the preferred graphics
    /// mode. Note that that is only a recommended mode by the OS image. Boot loader may choose a
    /// different mode if it sees fit.
    graphics: GraphicsRequest,
}

/// Compile time check that Header is of size 48. If the following line throws a size mismatch
/// compile error, then you know that sizeof `Header` is messed up!
const _: [(); 48] = [(); core::mem::size_of::<Header>()];

/// Optional part of the Multiboot header which includes load and entry addresses that override the
/// values from the ELF header.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadAddressRequest {
    /// Contains the address corresponding to the beginning of the Multiboot header — the physical
    /// memory location at which the magic value is supposed to be loaded. This field serves to
    /// synchronize the mapping between OS image offsets and physical memory addresses.
    header_addr: u32,

    /// Contains the physical address of the beginning of the text segment. The offset in the OS
    /// image file at which to start loading is defined by the offset at which the header was
    /// found, minus (header_addr - load_addr). load_addr must be less than or equal to
    /// header_addr.
    load_addr: u32,

    /// Contains the physical address of the end of the data segment. (load_end_addr - load_addr)
    /// specifies how much data to load. This implies that the text and data segments must be
    /// consecutive in the OS image; this is true for existing a.out executable formats. If this
    /// field is zero, the boot loader assumes that the text and data segments occupy the whole OS
    /// image file.
    load_end_addr: u32,

    /// Contains the physical address of the end of the bss segment. The boot loader initializes
    /// this area to zero, and reserves the memory it occupies to avoid placing boot modules and
    /// other data relevant to the operating system in that area. If this field is zero, the boot
    /// loader assumes that no bss segment is present.
    bss_end_addr: u32,

    /// The physical address to which the boot loader should jump in order to start running the
    /// operating system.
    entry_addr: u32,
}

/// Optional part of the Multiboot header which requests either a graphical framebuffer or a text
/// mode console from the bootloader.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsRequest {
    /// Contains 0 for linear graphics mode or 1 for EGA-standard text mode. Everything else is
    /// reserved for future expansion. Note that the boot loader may set a text mode even if this
    /// field contains 0, or set a video mode even if this field contains 1.
    mode: u32,

    /// Contains the number of the columns. This is specified in pixels in a graphics mode, and in
    /// characters in a text mode. The value zero indicates that the OS image has no preference.
    width: u32,

    /// Contains the number of the lines. This is specified in pixels in a graphics mode, and in
    /// characters in a text mode. The value zero indicates that the OS image has no preference.
    height: u32,

    /// Contains the number of bits per pixel in a graphics mode, and zero in a text mode. The
    /// value zero indicates that the OS image has no preference.
    depth: u32,
}

impl GraphicsRequest {
    pub const fn default_const() -> Self {
        Self {
            mode: 1,
            width: 0,
            height: 0,
            depth: 0,
        }
    }

    pub const fn with_framebuffer(self) -> Self {
        Self { mode: 0, ..self }
    }

    pub const fn with_text_mode(self) -> Self {
        Self { mode: 1, ..self }
    }
}

/// Builder struct to construct a valid multiboot header. The builder methods to enable specific
/// features.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct HeaderBuilder {
    flags: u32,
    addresses: Option<LoadAddressRequest>,
    graphics: Option<GraphicsRequest>,
}

impl HeaderBuilder {
    /// Create a new header builder with default flags. This could be replaced with
    /// Default::default() if it was const. But it isn't so plase use new() if you want a const
    /// object.
    pub const fn new() -> Self {
        Self {
            flags: 0,
            addresses: None,
            graphics: None,
        }
    }

    /// Requests that any modules loaded by the bootloader be aligned on page boundaries (4K).
    pub const fn request_aligned_modules(self) -> Self {
        Self {
            flags: self.flags | (1 << 0),
            ..self
        }
    }

    /// Requests a memory map from the bootloader by setting the corresponding header flag.
    pub const fn request_memory_map(self) -> Self {
        Self {
            flags: self.flags | (1 << 1),
            ..self
        }
    }

    /// Request graphics mode from the bootloader specified by `graphics`.
    pub const fn request_graphics(self, graphics: GraphicsRequest) -> Self {
        Self {
            flags: self.flags | (1 << 2),
            graphics: Some(graphics),
            ..self
        }
    }

    /// Request a graphical framebuffer and let the bootloader decide width, height and depth.
    pub const fn request_default_framebuffer(self) -> Self {
        self.request_graphics(GraphicsRequest::default_const().with_framebuffer())
    }

    /// Request EGA text mode graphics and let the bootloader decide columns and rows.
    pub const fn request_default_textmode(self) -> Self {
        self.request_graphics(GraphicsRequest::default_const().with_text_mode())
    }

    /// Build a valid multiboot header using the selected flags and compute the header checksum.
    pub const fn build(self) -> Header {
        // The `magic`, `flags` and `checksum` fields must have an unsigned sum of zero.
        const HEADER_MAGIC: u32 = 0x1bad_b002;
        Header {
            magic: HEADER_MAGIC,
            flags: self.flags,
            checksum: !HEADER_MAGIC.wrapping_add(self.flags) + 1,
            addresses: match self.addresses {
                Some(addresses) => addresses,
                None => LoadAddressRequest {
                    header_addr: 0,
                    load_addr: 0,
                    load_end_addr: 0,
                    bss_end_addr: 0,
                    entry_addr: 0,
                },
            },
            graphics: match self.graphics {
                Some(graphics) => graphics,
                None => GraphicsRequest::default_const(),
            },
        }
    }
}
