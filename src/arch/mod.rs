use core::arch::asm;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod io;

/// Disable interrupts and stop execution on this core indefinitely.
#[inline(always)]
pub fn halt_core() -> ! {
    irq_disable();
    loop {
        wait_for_irq();
    }
}

#[inline(always)]
fn wait_for_irq() {
    unsafe {
        asm!("hlt");
    }
}

#[inline(always)]
fn irq_disable() {
    unsafe {
        asm!("cli");
    }
}
