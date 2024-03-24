//! This module contains the structures used to implement the multiboot boot protocol. It provides
//! the kernel with parameters and information required to boot the system (e.g. system memory map,
//! the command line and pointers to loaded modules. Note that multiboot can only load 32-bit
//! kernels.
//!
//! This source file contains verbatim copies of the multiboot specification in version 0.6.96 for
//! which permission is granted to make and distribute verbatim copies provided the copyright notice
//! and this permission notice are preserved on all copies:
//!
//! * Copyright © 1995,96 Bryan Ford <baford@cs.utah.edu>
//! * Copyright © 1995,96 Erich Stefan Boleyn <erich@uruk.org>
//! * Copyright © 1999,2000,2001,2002,2005,2006,2009,2010 Free Software Foundation, Inc.

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
