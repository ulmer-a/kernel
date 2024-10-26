//! This crate makes it very easy for your rust kernel to boot using the multiboot protocol (v1).
//! For more information, have a look at the multiboot specification:
//!
//! <https://www.gnu.org/software/grub/manual/multiboot/multiboot.html> (version 0.6.96)

#![no_std]

pub mod header;
pub mod mmap;
pub mod module;

/// The kernel-facing abstraction to the Multiboot information structure.
#[derive(Clone)]
pub struct Multiboot<'mb> {
    inner: &'mb InnerMultiboot,
}

impl Multiboot<'_> {
    /// Check multiboot magic value and try to dereference pointer to information structure.
    ///
    /// ### Safety
    ///
    /// The multiboot pointer has to be aligned, non-null and must not be mutated during the `'mb`
    /// lifetime.
    pub unsafe fn from_addr<'mb>(magic: u32, mb_ptr: *const core::ffi::c_void) -> Multiboot<'mb> {
        // Check multiboot magic value and try to dereference pointer to information structure
        let mb_ptr = mb_ptr.cast::<InnerMultiboot>();
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

    /// Returns the kernel command line if it has been passed by the bootloader and is valid.
    pub fn command_line(&self) -> Option<&str> {
        self.inner.command_line().map(|c_str| c_str.to_str())?.ok()
    }

    /// Returns the bootloader name if it has been passed by the bootloader and is valid.
    pub fn bootloader_name(&self) -> Option<&str> {
        self.inner
            .boot_loader_name()
            .map(|c_str| c_str.to_str())?
            .ok()
    }

    /// Returns an iterator that can be used to traverse the memory map passed by the bootloader,
    /// or `None` if there is no memory map present.
    pub fn memory_map(&self) -> Option<mmap::MemoryMapIter> {
        self.inner.memory_map()
    }

    /// Returns a reference to the array of modules passed by the bootloader, if present.
    pub fn modules(&self) -> Option<&[module::Module]> {
        self.inner.modules()
    }

    /// Returns the framebuffer information if it has been passed by the bootloader and is valid.
    pub fn framebuffer(&self) -> Option<Framebuffer> {
        self.inner.framebuffer().map(|fbr| fbr.clone())
    }
}

impl<'mb> From<&'mb InnerMultiboot> for Multiboot<'mb> {
    fn from(inner: &'mb InnerMultiboot) -> Self {
        Self { inner }
    }
}

impl core::fmt::Debug for Multiboot<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Multiboot")
            .field("flags", &self.inner.flags)
            // .field("low_mem", &self.inner.mem_lower)
            // .field("high_mem", &self.inner.mem_upper)
            .field("cmdline", &self.inner.command_line())
            .field("bootloader", &self.inner.boot_loader_name())
            .field("mmap", &self.memory_map())
            .field("vbe", &self.inner.vbe())
            .field("framebuffer", &self.inner.framebuffer())
            .finish_non_exhaustive()
    }
}

/// When the bootloader (e.g. GRUB) transfers control to the kernel, an instance of this struct is
/// passed along to the kernel. It contains information vital to the kernel startup procedure.
#[repr(C)]
#[derive(Debug)]
struct InnerMultiboot {
    /// Indicates the presence and validity of other fields in the Multiboot information structure.
    /// Any set bits that the operating system does not understand should be ignored.
    flags: Flags,

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
    /// the [`module::Module`] structure.
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
    mmap_addr: u32,

    drives_length: u32,

    drives_addr: u32,

    config_table: u32,

    /// If bit 9 in the `flags` is set, the `boot_loader_name` field is valid, and contains the
    /// physical address of the name of a boot loader booting the kernel. The name is a normal
    /// C-style zero-terminated string.
    boot_loader_name: u32,

    apm_table: u32,

    /// VBE table is available if bit 11 of `flags` is set.
    vbe: Vbe,

    /// Framebuffer table is available if bit 12 of `flags` is set.
    framebuffer: Framebuffer,
}

// Compile time check for sizeof(InnerMultiboot) == 116
const _: [(); 116] = [(); core::mem::size_of::<InnerMultiboot>()];

impl InnerMultiboot {
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
    pub fn modules(&self) -> Option<&[module::Module]> {
        let mods_ptr = self.mods_addr as *const module::Module;
        if self.flags.is_modules_valid() & !mods_ptr.is_null() {
            Some(unsafe { core::slice::from_raw_parts(mods_ptr, self.mods_count as usize) })
        } else {
            None
        }
    }

    /// Returns an iterator that can be used to traverse the memory map passed by the bootloader,
    /// or `None` if there is no memory map present.
    pub fn memory_map<'mb>(&'mb self) -> Option<mmap::MemoryMapIter<'mb>> {
        let mmap_ptr = self.mmap_addr as *const u8;
        if self.flags.is_mmap_valid() && !mmap_ptr.is_null() {
            let mmap_buffer = unsafe {
                // SAFETY: We just checked that the memory map is present and the pointer to its
                // memory is non-null. Also, we explicitly make sure that the lifetime of the
                // resulting reference is tied to the lifetime of the Multiboot struct.
                core::slice::from_raw_parts::<'mb>(mmap_ptr, self.mmap_length as usize)
            };
            Some(mmap_buffer.into())
        } else {
            None
        }
    }

    fn boot_loader_name(&self) -> Option<&core::ffi::CStr> {
        let bootloader_name_ptr = self.boot_loader_name as *const core::ffi::c_char;
        if self.flags.is_bootloader_name_valid() && !bootloader_name_ptr.is_null() {
            Some(unsafe { core::ffi::CStr::from_ptr(bootloader_name_ptr) })
        } else {
            None
        }
    }

    fn vbe(&self) -> Option<&Vbe> {
        if self.flags.is_nth_bit_set(11) {
            Some(&self.vbe)
        } else {
            None
        }
    }

    fn framebuffer(&self) -> Option<&Framebuffer> {
        if self.flags.is_framebuffer_valid() {
            Some(&self.framebuffer)
        } else {
            None
        }
    }
}

#[derive(Clone)]
#[repr(transparent)]
struct Flags(u32);

impl Flags {
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

    fn is_bootloader_name_valid(&self) -> bool {
        self.is_nth_bit_set(9)
    }

    fn is_framebuffer_valid(&self) -> bool {
        self.is_nth_bit_set(12)
    }
}

impl core::fmt::Debug for Flags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{{ ")?;
        for flag_name in Self::bits()
            .iter()
            .filter_map(|(name, bit)| self.is_nth_bit_set(*bit).then_some(name))
        {
            write!(f, "{flag_name}, ")?;
        }
        write!(f, ".. }}")
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
struct Vbe {
    control_info: u32,
    mode_info: u32,
    mode: u16,
    interface_seg: u16,
    interface_off: u16,
    interface_len: u16,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C, packed)]
pub struct Framebuffer {
    addr: u64,
    pitch: u32,
    width: u32,
    height: u32,
    bits_per_pixel: u8,
    framebuffer_type: u8,
    color_info: [u8; 6],
}
