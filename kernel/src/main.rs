#![no_std]
#![no_main]
#![feature(naked_functions)] // boot::_multiboot_entry()
#![feature(slice_from_ptr_range)] // mem::bss()
#![feature(allocator_api)]
#![deny(clippy::float_arithmetic, reason = "no hardware float support")]
#![deny(clippy::pedantic)]
#![deny(clippy::unwrap_used)]

extern crate alloc;

mod arch;
mod boot;
mod logging;
mod mem;

/// The panic handler is called whenever the kernel encountered an unrecoverable error. It's purpose
/// is to halt the system and report debug information to the user.
#[panic_handler]
fn panic(reason: &core::panic::PanicInfo) -> ! {
    log::error!("Halting due to unrecoverable kernel panic:\n{}", reason);
    arch::halt_core();
}
