//! Memory management

use physical::MemoryMap;

pub mod fmt;
pub mod physical;

/// Max size of physical memory direct mapping on 32-bit x86 (virtual address space size limit).
#[cfg(target_arch = "x86")]
pub const PHYS_MAP_LIMIT: u64 = 0x0800_0000; // 128 MiB

pub fn bootstrap_subsystem(memory_map: impl MemoryMap) {
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
