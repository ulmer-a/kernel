//! Kernel entry point and boot protocol implementation(s). Usually, the bootloader (e.g. GRUB)
//! loads the kernel into memory and jumps to it. The boot protocol (e.g. multiboot) defines the
//! machine state at that point and any other information that the bootloader passes to the kernel.

mod multiboot;

/// Instance of the multiboot header in static memory. It is placed in the `.multiboot` section of
/// the binary so that it can be linked into the first 8K of the binary via the linker script.
///
/// More details: [multiboot::Header]
#[used]
#[link_section = ".multiboot"]
static MULTIBOOT_HEADER: multiboot::Header = multiboot::Header::new()
    .request_aligned_modules()
    .request_memory_map();

/// The base address of the boot stack. The stack grows downwards from this address.
const BOOT_STACK_BASE: usize = 0x8_0000;

/// Entry point into the kernel in case the `multiboot` boot protocol is being used. This will
/// prepare the processor to run Rust code before it jumps to the [`multiboot_start()`] function.
#[naked]
#[no_mangle]
unsafe extern "C" fn _multiboot_entry() {
    // Exact machine state at this point is defined by the multiboot specification.
    // * `eax`: Must contain magic value `0x2BADB002`.
    // * `ebx`: Contains the physical address of the multiboot information structure.
    // * `esp`: Stack pointer is in an undefined state. We must load our own.
    core::arch::asm!(
        "mov ${stack_ptr}, %esp",
        "push %ebx", // 2nd argument to `multiboot_start()`
        "push %eax", // 1st argument to `multiboot_start()`
        "call clear_bss",
        "call multiboot_start",
        stack_ptr = const { BOOT_STACK_BASE },
        options(att_syntax, noreturn)
    );
}

/// Rust entry point into the kernel after a stack has been setup.
#[no_mangle]
extern "C" fn multiboot_start(magic: u32, mb_ptr: *const multiboot::BootInfo) -> ! {
    use log::{debug, info};

    crate::logging::initialize_kernel_log();
    info!("Kernel by Alexander Ulmer v{}", env!("CARGO_PKG_VERSION"));
    info!("Copyright 2017-2024");

    // Check information structure pointer as well as the magic value to make sure that indeed we
    // should use the `multiboot` protocol.
    assert!(!mb_ptr.is_null(), "Checking multiboot pointer");
    assert_eq!(magic, 0x2badb002, "Checking multiboot magic value");
    debug!("Valid `multiboot` signature found: struct @ {:?}", mb_ptr);

    let multiboot = unsafe { &*mb_ptr };

    // Print command line to kernel log
    info!(
        "Command line: {}",
        match multiboot.command_line() {
            Some(cmdline) => cmdline.to_str().unwrap_or("invalid (non-utf-8)"),
            None => "none",
        },
    );

    // Print memory map to kernel log
    debug!("Memory map:");
    for mem_chunk in multiboot.memory_map().expect("Ain't got no mmmap") {
        debug!("├─ {}", mem_chunk);
    }
    debug!("└─ total: XXX");

    // TODO Implement the rest of the boot process here.
    crate::arch::halt_core();
}

/// Clears the entire BSS segment of the kernel image. This may corrupt kernel memory if the
/// function is executed after data in the BSS segment has been mutated. Furthermore, this function
/// assumes that the symbols `__bss_start` and `_bss_end` defined in the linker script are valid and
/// well-aligned addresses.
#[no_mangle]
unsafe extern "C" fn clear_bss() {
    // Symbols defined by linker script:
    extern "C" {
        /// Start address of the BSS segment.
        static __bss_start: u8;

        /// End address of the BSS segment.
        static __bss_end: u8;
    }

    unsafe {
        core::slice::from_mut_ptr_range(core::ops::Range {
            start: (&__bss_start as *const u8).cast_mut(),
            end: (&__bss_end as *const u8).cast_mut(),
        })
        .fill(0);
    }
}
