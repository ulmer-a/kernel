//! Boot protocol implementation(s). Usually, the bootloader (e.g. GRUB) loads the kernel into
//! memory and jumps to it. A boot protocol (e.g. multiboot) is used to define the machine state at
//! the point where the kernel is entered.

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

/// Entry point into the kernel in case the `multiboot` boot protocol is being used. The multiboot
/// specification defines the machine state at this point:
///
/// * `eax`: Must contain `0x2BADB002`. This value indicates to the kernel that it was loaded by a
/// multiboot-compliant bootloader.
/// * `ebx`: Contains the 32-bit physical address of the multiboot information structure provided
/// by the boot loader.
/// * `cs`: Contains valid read/exec code segment selector with an offset of `0` and a limit of
/// `0xffff_ffff`.
/// * `ds`, `es`, `fs`, `gs`: Contain valid read/write data segment selectors with an offset of `0`
/// and a limit of `0xffff_ffff`.
/// * `cr0`: PG (paging) disabled, PE (protected mode) enabled
/// * `eflags`: VM clear, IF clear (interrupts disabled)
/// * `esp`: Undefined. We need to load our own stack.
/// * `gdtr`: Undefined. We can't load segment registers until we install our own global descriptor
/// table.
/// * `idtr`:   Undefined. We must install our own interrupt descriptor table.
#[naked]
#[no_mangle]
unsafe extern "C" fn _multiboot_entry() {
    core::arch::asm!(
        // Bootloader leaves `%esp` undefined. We must setup our own stack.
        "mov ${stack_ptr}, %esp",
        // Call `multiboot_start()` function and pass on the magic value and multiboot struct
        // pointer left behind by the bootloader.
        "push %ebx", // 2nd argument
        "push %eax", // 1st argument
        "call multiboot_start",
        stack_ptr = const { BOOT_STACK_BASE },
        options(att_syntax, noreturn)
    );
}

#[no_mangle]
extern "C" fn multiboot_start(magic: u32, mb_ptr: *const multiboot::BootInfo) -> ! {
    use log::{debug, info};

    // Safety: This must be done before any relevant statics are accessed.
    unsafe {
        clear_bss();
    }

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

    // TODO Implement the rest of the boot process here.
    crate::arch::halt_core();
}

/// Clears the entire BSS segment of the kernel image. This may corrupt kernel memory if the
/// function is executed after data in the BSS segment has been mutated. Furthermore, this function
/// assumes that the symbols `__bss_start` and `_bss_end` defined in the linker script are valid and
/// well-aligned addresses.
pub unsafe fn clear_bss() {
    // Symbols defined by linker script:
    extern "C" {
        /// Start address of the BSS segment.
        static __bss_start: u8;

        /// End address of the BSS segment.
        static __bss_end: u8;
    }

    core::slice::from_mut_ptr_range(core::ops::Range {
        start: (&__bss_start as *const u8).cast_mut(),
        end: (&__bss_end as *const u8).cast_mut(),
    })
    .fill(0);
}
