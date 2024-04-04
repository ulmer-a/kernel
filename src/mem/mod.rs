//! Memory management
//!
//! This module needs to provide the kernel with the abilities to manage and allocate phyiscal and
//! virtual memory. Requirements include but are not limited to:
//!
//! * Allocating single physical page frames
//! * Allocating kernel heap memory (in various sizes and alignments)
//! * Allocating sets of contiguous physical page frames below 16MB (for ISA DMA) or below 4GB (for
//!   PCI busmastering DMA)
//! * Translating virtual to physical addresses

use core::fmt::{Display, Formatter, Result};

pub mod physical;

/// Max size of physical memory direct mapping on 32-bit x86 (virtual address space size limit).
#[cfg(target_arch = "x86")]
pub const PHYS_MAP_LIMIT: u64 = 0x0800_0000; // 128 MiB

pub fn bootstrap_subsystem(memory_map: impl Iterator<Item = physical::MemoryChunk> + Clone) {
    // Print system memory map to the kernel log
    print_memory_map(memory_map.clone());

    // Find a usable memory range above 32 MiB (so it doesn't interfere with the kernel binary and
    // modules) and below `PHYS_MAP_LIMIT`. This will be used temporarily to allocate pages
    let tmp_allocator_memory = memory_map
        .filter(|chunk| chunk.is_usable())
        .filter_map(|chunk| chunk.crop(0x0200_0000, PHYS_MAP_LIMIT))
        .last()
        .expect("Cannot find a suitable chunk of temporary boot memory.");

    log::debug!("Boot memory: {}", tmp_allocator_memory);

    // TODO
    // 1. Implement and initialise simple page frame allocator.
    // 2. Implement boot page table mapper. If possible, use large pages.
    //   a) ident map all available chunks up to 3GiB.
    //   b) direct map all available chunks up to 128MiB to 3.5 GiB.
    //   c) map kernel binary at just below 4GiB.
    // 3. Implement the slab allocator.
    // 4. Implement and setup the buddy allocators.
    // 5. (Optional) Implement and setup the fast stack allocator.
    // 6. Implement the kernel heap.
    // 8. Move all data which needs to be kept into the kernel heap.
    // 7. Move kernel and its stack to the high half + rewind stack!
}

/// Prints the bootloader-provided memory map to the kernel log.
fn print_memory_map(memory_map: impl Iterator<Item = physical::MemoryChunk>) {
    log::info!("Bootloader-provided memory map:");

    let total_bytes_available = memory_map
        .map(|chunk| {
            log::info!("├─ {}", chunk);
            if chunk.is_usable() {
                chunk.length
            } else {
                0
            }
        })
        .sum::<u64>();

    log::info!(
        "└─ total memory available: {}",
        total_bytes_available.fmt_as_bytes()
    );
}

pub trait ByteLength {
    fn in_gigabytes(&self) -> f32 {
        self.in_megabytes() / 1024.0
    }

    fn in_megabytes(&self) -> f32 {
        self.in_kilobytes() / 1024.0
    }

    fn in_kilobytes(&self) -> f32 {
        self.in_bytes() as f32 / 1024.0
    }

    fn in_bytes(&self) -> u64;

    fn fmt_as_bytes(self) -> ByteSizeFormatter<Self>
    where
        Self: Sized,
    {
        ByteSizeFormatter(self)
    }
}

pub struct ByteSizeFormatter<T: ByteLength>(T);

impl<T: ByteLength> Display for ByteSizeFormatter<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.0.in_bytes() >= 0x1_0000_0000 {
            // >= 4 GiB
            write!(f, "{:.1} GiB", self.0.in_gigabytes())
        } else if self.0.in_bytes() >= 0x0080_0000 {
            // >= 8 MiB
            write!(f, "{:.1} MiB", self.0.in_megabytes())
        } else if self.0.in_bytes() >= 0x2000 {
            // >= 8 KiB
            write!(f, "{:.1} KiB", self.0.in_kilobytes())
        } else {
            write!(f, "{} B", self.0.in_bytes())
        }
    }
}

impl ByteLength for u64 {
    fn in_bytes(&self) -> u64 {
        *self
    }
}
