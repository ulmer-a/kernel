//! This crate contains the structures used to implement the multiboot boot protocol as defined in
//! the corresponding specification:
//!
//! https://www.gnu.org/software/grub/manual/multiboot/multiboot.html (version 0.6.96)

#![no_std]

mod header;
mod mmap;
mod module;

pub use header::*;
pub use mmap::*;
use module::Module;

#[derive(Clone)]
pub struct BootInfo<'mb> {
    inner: &'mb InnerBootInfo,
}

impl BootInfo<'_> {
    /// Check multiboot magic value and try to dereference pointer to information structure.
    ///
    /// ### Safety
    ///
    /// The multiboot pointer has to be aligned, non-null and must not be mutated during the `'mb`
    /// lifetime.
    pub unsafe fn from_addr<'mb>(magic: u32, mb_ptr: *const core::ffi::c_void) -> BootInfo<'mb> {
        // Check multiboot magic value and try to dereference pointer to information structure
        let mb_ptr = mb_ptr.cast::<InnerBootInfo>();
        assert_eq!(magic, 0x2badb002, "Multiboot magic value mismatch");
        assert!(mb_ptr.is_aligned(), "Multiboot pointer must be aligned");
        unsafe {
            // Safety: Checked for alignment
            mb_ptr
                .as_ref::<'mb>()
                .expect("Multiboot information structure pointer should be non-null")
        }
        .into()
    }
}

impl<'mb> From<&'mb InnerBootInfo> for BootInfo<'mb> {
    fn from(inner: &'mb InnerBootInfo) -> Self {
        Self { inner }
    }
}

impl core::ops::Deref for BootInfo<'_> {
    type Target = InnerBootInfo;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl core::fmt::Debug for BootInfo<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Multiboot")
            .field("flags", &self.inner.flags)
            // .field("low_mem", &self.inner.mem_lower)
            // .field("high_mem", &self.inner.mem_upper)
            .field("cmdline", &self.inner.command_line())
            .field("mmap", &self.memory_map())
            .finish_non_exhaustive()
    }
}

/// When the bootloader (e.g. GRUB) transfers control to the kernel, an instance of this struct is
/// passed along to the kernel. It contains information vital to the kernel startup procedure.
#[repr(C)]
#[derive(Debug)]
pub struct InnerBootInfo {
    /// Indicates the presence and validity of other fields in the Multiboot information structure.
    /// Any set bits that the operating system does not understand should be ignored.
    flags: BootInfoFlags,

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

impl InnerBootInfo {
    /// Returns the kernel command line if it has been passed by the bootloader and is valid.
    pub fn command_line(&self) -> Option<&core::ffi::CStr> {
        let cmdline_ptr = self.cmdline as *const core::ffi::c_char;
        if self.flags.is_cmdline_valid() && !cmdline_ptr.is_null() {
            Some(unsafe { core::ffi::CStr::from_ptr(cmdline_ptr) })
        } else {
            None
        }
    }

    /// Returns a reference to the array of modules passed by the bootloader, if present.
    pub fn modules(&self) -> Option<&[Module]> {
        let mods_ptr = self.mods_addr as *const Module;
        if self.flags.is_modules_valid() & !mods_ptr.is_null() {
            Some(unsafe { core::slice::from_raw_parts(mods_ptr, self.mods_count as usize) })
        } else {
            None
        }
    }

    /// Returns an iterator that can be used to traverse the memory map passed by the bootloader,
    /// or `None` if there is no memory map present.
    pub fn memory_map<'mb>(&'mb self) -> Option<mmap::MemoryMapIter<'mb>> {
        use core::slice;

        let mmap_ptr = self.mmap as *const u8;
        if self.flags.is_mmap_valid() && !mmap_ptr.is_null() {
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

#[derive(Clone)]
#[repr(transparent)]
struct BootInfoFlags(u32);

impl BootInfoFlags {
    const fn bits() -> &'static [(&'static str, usize)] {
        &[
            ("MEM", 0),
            ("BOOTDEV", 1),
            ("CMDLINE", 2),
            ("MODS", 3),
            ("SYMBOLS", 4),
            ("ELFSHT", 5),
            ("MMAP", 6),
            ("DRV", 7),
            ("CFG", 8),
            ("BLDR", 9),
            ("APM", 10),
            ("VBE", 11),
            ("FBR", 12),
        ]
    }

    fn is_nth_bit_set(&self, bit: usize) -> bool {
        self.0 & (1 << bit) as u32 != 0
    }

    fn is_cmdline_valid(&self) -> bool {
        self.is_nth_bit_set(2)
    }

    fn is_mmap_valid(&self) -> bool {
        self.is_nth_bit_set(3)
    }

    fn is_modules_valid(&self) -> bool {
        self.is_nth_bit_set(6)
    }
}

impl core::fmt::Debug for BootInfoFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{{ ")?;
        for flag_set in Self::bits()
            .iter()
            .filter_map(|(name, bit)| self.is_nth_bit_set(*bit).then_some(name))
        {
            write!(f, "{flag_set}, ")?;
        }
        write!(f, ".. }}")
    }
}
