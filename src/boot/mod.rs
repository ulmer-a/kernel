//! ## Boot procedure implementation
//!
//! A dedicated bootloader (e.g. GRUB) must be used to load the kernel image into memory and pass
//! control to it. The bootloader must also provide the kernel with information about the machine
//! and its configuration (e.g. memory map, command line arguments, etc.). The modalities of these
//! tasks are defined by the boot protocol. On the x86-32 architecture, this kernel uses the
//! `multiboot` boot protocol. Please check the specification for details on how it works.

mod multiboot;

/// Instance of the multiboot header in static memory. It is used to tell the bootloader which
/// features the kernel requires from it. The header is placed in the `.multiboot` section of the
/// the binary so that it can be linked into the first 8K of the binary as this is required by the
/// specification.
///
/// More details: [multiboot::Header]
#[used]
#[link_section = ".multiboot"]
#[cfg(target_arch = "x86")]
static MULTIBOOT_HEADER: multiboot::Header = multiboot::Header::new()
    .request_aligned_modules()
    .request_memory_map();

/// The top address of the boot stack. The stack grows downwards from this address.
const BOOT_STACK_BASE: usize = 0x8_0000;

/// The entry point is the first code that gets executed once the bootloader passes control to the
/// kernel. There are multiple way to tell the bootloader about its location. Currently, we don't
/// enable any special fields in the [`MULTIBOOT_HEADER`] so the bootloader will just jump to the
/// address specificed in the entry point field of the the ELF file. For this to work, the linker
/// script needs to be written in a way that the address of the [`_multiboot_entry()`] function
/// actually ends up in the entry point field of the ELF file.
///
/// Before jumping to the [`multiboot_main()`] function, this function will perform the following
/// tasks:
///
/// 1. Setup a stack by loading the `esp` register with the top address of the kernel stack.
/// 2. Save the pointer to the multiboot information structure found in the `ebx` register.
/// 3. Save the multiboot magic value found in the `eax` register.
/// 4. Call the [`clear_bss()`] function.
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
extern "C" fn multiboot_main(magic: u32, mb_ptr: *const multiboot::BootInfo) -> ! {
    use log::{debug, info};

    crate::logging::initialize_kernel_log();
    info!("Kernel by Alexander Ulmer v{}", env!("CARGO_PKG_VERSION"));
    info!("Copyright 2017-2024");

    // Check multiboot magic value and try to dereference pointer to information structure
    assert_eq!(magic, 0x2badb002, "Multiboot magic value mismatch");
    let multiboot = unsafe {
        mb_ptr
            .as_ref()
            .expect("Multiboot information structure pointer should be non-null")
    };

    debug!("Multiboot structure @ {:?}", mb_ptr);
    debug!("Multiboot dump: {:?}", multiboot);

    // Retrieve multiboot memory map and use it to bootstrap the memory subsystem
    let memory_map = multiboot
        .memory_map()
        .expect("Expected multiboot memory map to be present");
    crate::mem::bootstrap_subsystem(memory_map);

    // TODO Implement the rest of the boot process here.
    crate::arch::halt_core();
}

/// Clears the entire BSS segment of the kernel image. This may corrupt kernel memory if the
/// function is executed after data in the BSS segment has been mutated. Furthermore, this function
/// assumes that the symbols `__bss_start` and `_bss_end` defined in the linker script are valid and
/// well-aligned addresses.
#[no_mangle]
unsafe extern "C" fn clear_bss() {
    use core::{ops::Range, slice};

    // Symbols defined by linker script:
    extern "C" {
        /// Start address of the BSS segment.
        static __bss_start: u8;

        /// End address of the BSS segment.
        static __bss_end: u8;
    }

    unsafe {
        slice::from_mut_ptr_range(Range {
            start: (&__bss_start as *const u8).cast_mut(),
            end: (&__bss_end as *const u8).cast_mut(),
        })
        .fill(0);
    }
}
