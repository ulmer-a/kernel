//! Memory management

mod heap;
pub mod paging;
pub mod physical;

use paging::PagingMode;

/// Max size of physical memory direct mapping on 32-bit x86 (virtual address space size limit).
#[cfg(target_arch = "x86")]
pub const PHYS_MAP_LIMIT: u64 = 0x0800_0000; // 128 MiB

pub fn bootstrap_subsystem<P: PagingMode>(memory_map: impl physical::MemoryMap) {
    // Print system memory map to the kernel log
    log::info!("System memory map:\n{}", memory_map.fmt());

    // Find a usable memory range above 32 MiB (so it doesn't interfere with the kernel binary and
    // modules) and below `PHYS_MAP_LIMIT`. This will be used temporarily to allocate pages
    let tmp_allocator_memory = memory_map
        .filter_usable()
        .filter_map(|chunk| chunk.crop(0x0200_0000, PHYS_MAP_LIMIT))
        .last()
        .expect("Cannot find a suitable chunk of temporary boot memory.");

    log::debug!("Boot memory: {}", tmp_allocator_memory);

    // Create virtual address space
    let _ = P::create_kernel_space();
    // TODO: identity map physical memory and setup heap

    // 1. Setup bootmem/memblock like allocator for further initialisation
    // 2. Setup buddy page frame allocator
    // 3. Implement virtual memory management
    //   a) ident map all available chunks up to 3GiB.
    //   b) direct map all available chunks up to 128MiB to 3.5 GiB.
    //   c) map kernel binary at just below 4GiB.
    // 6. Implement the kernel heap.
    // 8. Move all data which needs to be kept into the kernel heap.
    // 7. Move kernel and its stack to the high half + rewind stack!
}
