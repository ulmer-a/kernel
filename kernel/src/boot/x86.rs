//! On the x86-32 architecture, this kernel uses the `multiboot` boot protocol. Please check the
//! specification for details on how it works.

use core::ops::Range;
use multiboot::{
    header::{Header as MultibootHeader, HeaderBuilder},
    Multiboot,
};

// use super::{__bss_end, __data_end, __text_start};

/// The top address of the boot stack. The stack grows downwards from this address.
const BOOT_STACK_BASE: usize = 0x8_0000;

/// Multiboot specification requires multiboot header to be present in the first 8K of the kernel
/// binary for the bootloader to search for. It signals to the bootloader that the kernel is
/// multiboot-compliant. Also, the kernel can request features from the bootloader via flags.
#[used]
#[link_section = ".multiboot"]
#[cfg(target_arch = "x86")]
static MULTIBOOT_HEADER: MultibootHeader = HeaderBuilder::new()
    .request_aligned_modules()
    .request_memory_map()
    // .request_load_addrs(LoadAddressRequest {
    //     header_addr: &MULTIBOOT_HEADER as *const MultibootHeader,
    //     load_addr: unsafe { &__text_start } as *const core::ffi::c_void,
    //     load_end_addr: unsafe { &__data_end } as *const core::ffi::c_void,
    //     bss_end_addr: unsafe { &__bss_end } as *const core::ffi::c_void,
    //     entry_addr: multiboot_start as *const (),
    // })
    .request_default_framebuffer()
    .build();

/// The entry point is the first code that gets executed once the bootloader passes control to the
/// kernel. There are multiple way to tell the bootloader about its location. Currently, we don't
/// enable any special fields in the [`MULTIBOOT_HEADER`] so the bootloader will just jump to the
/// address specificed in the entry point field of the the ELF file. For this to work, the linker
/// script needs to be written in a way that the address of the [`multiboot_start()`] function
/// actually ends up in the entry point field of the ELF file.
///
/// Before jumping to the [`multiboot_main()`] function, this function will perform the following
/// tasks:
///
/// 1. Setup a stack by loading the `esp` register with the top address of the kernel stack.
/// 2. Save the pointer to the multiboot information structure found in the `ebx` register.
/// 3. Save the multiboot magic value found in the `eax` register.
/// 4. Call the [`super::clear_bss()`] function.
/// 5. Call the [`multiboot_main()`] function while passing both of the previously saved values as
///    arguments.
#[naked]
#[no_mangle]
#[cfg(target_arch = "x86")]
unsafe extern "C" fn multiboot_start() {
    // Exact machine state at this point is defined by the multiboot specification.
    // * `eax`: Must contain magic value `0x2BADB002`.
    // * `ebx`: Contains the physical address of the multiboot information structure.
    // * `esp`: Stack pointer is in an undefined state. We must load our own.
    core::arch::naked_asm!(
        "mov ${stack_ptr}, %esp",
        "push %ebx",
        "push %eax",
        "call clear_bss",
        "call multiboot_main",
        stack_ptr = const { BOOT_STACK_BASE },
        options(att_syntax)
    );
}

/// Coming from [`multiboot_start()`], this is the first true Rust code that gets executed after
/// the bootloader passes control to the kernel. Its tasks are:
///
/// 1. Initialize the kernel log.
/// 2. Verify the multiboot magic value and information structure pointer.
/// 3. Initialize the memory subsystem based on the memory map provided by the bootloader via the
///    multiboot information structure.
#[no_mangle]
#[cfg(target_arch = "x86")]
extern "C" fn multiboot_main(magic: u32, mb_ptr: *const core::ffi::c_void) -> ! {
    use log::{debug, info};
    use types::mem::RangeFilterExt;

    crate::logging::initialize_kernel_log();
    info!("Kernel by Alexander Ulmer v{}", env!("CARGO_PKG_VERSION"));
    info!("Copyright 2017-2024");

    let multiboot = unsafe {
        // Safety: Memory must not be mutated.
        Multiboot::from_addr(magic, mb_ptr)
    };

    // Dump entire multiboot structure when in debug mode
    #[cfg(debug_assertions)]
    debug!(
        "Bootloader-provided struct @ {:?}: {:#?}",
        mb_ptr, multiboot
    );

    // Retrieve multiboot memory map and split it at 16 MiB. Everything below 16 MiB we call
    // reclaimable, because it is clobbered by the bootloader, BIOS, the kernel image, etc.
    // We assume that everything above 16 MiB is available and safe to use. However, this split
    // implies that at least 16 MiB of system RAM is available.
    let (reclaimable, available_mem) = multiboot
        .memory_map()
        .expect("Need multiboot memory map to be present")
        .map(Into::into)
        .split_once(0x100_0000); // 16 MiB

    debug!("Reclaimable boot memory: {:?}", reclaimable);
    debug!("Remaining memory: {:?}", available_mem.clone());

    crate::mem::bootstrap_subsystem(available_mem);

    // TODO Implement the rest of the boot process here.

    crate::arch::halt_core();
}

use crate::mem::paging::PhysicalPageNumber;

/// Allocates by incrementing the start page. This allocator cannot deallocate.
pub struct SimplePageFrameAllocator {
    range: Range<PhysicalPageNumber>,
}

impl SimplePageFrameAllocator {
    pub fn new(range: Range<PhysicalPageNumber>) -> Self {
        Self { range }
    }

    pub fn alloc_page(&mut self) -> Option<PhysicalPageNumber> {
        if self.range.start < self.range.end {
            let ppn = self.range.start;
            self.range.start += 1;
            Some(ppn)
        } else {
            None
        }
    }
}
