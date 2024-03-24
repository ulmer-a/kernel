#![no_std]
#![no_main]

// extern crate alloc;

#[no_mangle]
extern "C" fn _multiboot_entry() {
    loop {}
}

/// The panic handler is called whenever the kernel encountered an unrecoverable error. It's purpose
/// is to halt the system and report debug information to the user.
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
