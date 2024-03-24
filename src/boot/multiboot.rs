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

// Only compile this module on `x86` (IA-32) architecture
#![cfg(target_arch = "x86")]

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
/// passed along to the kernel. It contains essential information with respect to the boot process
/// (memory map, command line).
#[repr(C, packed)]
pub struct BootInfo {
    /// Indicates the presence and validity of other fields in the Multiboot information structure.
    /// All as-yet-undefined bits must be set to zero by the boot loader. Any set bits that the
    /// operating system does not understand should be ignored. Thus, the `flags` field also
    /// functions as a version indicator, allowing the Multiboot information structure to be
    /// expanded in the future without breaking anything.
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
    /// which BIOS disk device the boot loader loaded the OS image from. If the OS image was not
    /// loaded from a BIOS disk, then this field must not be present (bit 3 must be clear). The
    /// operating system may use this field as a hint for determining its own root device, but is
    /// not required to. The `boot_device` field is laid out in four one-byte subfields as follows:
    ///
    /// ```text
    /// +-------+-------+-------+-------+
    /// | part3 | part2 | part1 | drive |
    /// +-------+-------+-------+-------+
    /// Least significant             Most significant
    /// ```
    ///
    /// The most significant byte contains the BIOS drive number as understood by the BIOS INT 0x13
    /// low-level disk interface: e.g. 0x00 for the first floppy disk or 0x80 for the first hard
    /// disk.
    //
    /// The three remaining bytes specify the boot partition. `part1` specifies the top-level
    /// partition number, `part2` specifies a sub-partition in the top-level partition, etc.
    /// Partition numbers always start from zero. Unused partition bytes must be set to 0xFF. For
    /// example, if the disk is partitioned using a simple one-level DOS partitioning scheme, then
    /// `part1` contains the DOS partition number, and `part2` and `part3` are both 0xFF. As
    /// another example, if a disk is partitioned first into DOS partitions, and then one of
    /// those DOS partitions is subdivided into several BSD partitions using BSD's "disk label"
    /// strategy, then `part1` contains the DOS partition number, `part2` contains the BSD
    /// sub-partition within that DOS partition, and `part3` is 0xFF.
    ///
    /// DOS extended partitions are indicated as partition numbers starting from 4 and increasing,
    /// rather than as nested sub-partitions, even though the underlying disk layout of extended
    /// partitions is hierarchical in nature. For example, if the boot loader boots from the second
    /// extended partition on a disk partitioned in conventional DOS style, then `part1` will be 5,
    /// and `part2` and `part3` will both be 0xFF.
    _boot_device: u32,

    /// If bit 2 of the `flags` word is set, the `cmdline` field is valid, and contains the
    /// physical address of the command line to be passed to the kernel. The command line is a
    /// normal C-style zero-terminated string. The exact format of command line is left to OS
    /// developers.
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
    mods_addr: *const core::ffi::c_void,

    _unused2: [u32; 4],

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
    /// Returns the kernel command line if it has been passed along by the bootloader.
    pub fn command_line(&self) -> Option<&core::ffi::CStr> {
        const COMMAND_LINE_PRESENT: u32 = 1 << 2;
        if self.flags & COMMAND_LINE_PRESENT != 0 && !self.cmdline.is_null() {
            return Some(unsafe { core::ffi::CStr::from_ptr(self.cmdline) });
        }
        None
    }
}
