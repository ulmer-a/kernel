use core::fmt::{Display, Formatter, Result};
use types::mem::MemoryChunk;

struct _PhysicalMemory {
    /// Buddy allocator for contiguous ranges of physical page frames below 16 MiB. Used to
    /// allocate ISA DMA buffers.
    isa_allocator: (),

    /// Buddy allocator for contiguous ranges of physical page frames from 16 MiB to 128 MiB. Used
    /// to allocate PCI busmastering DMA buffers as well as page table pages (these also need to be
    /// accessible via virtual mappings).
    pci_allocator: (),

    /// Stack-based allocator to quickly allocate single page frames. Used for everything else. The
    /// content of these page frames cannot be accessed without being mapped into an address space.
    highmem_allocator: (),
}


pub trait MemoryMap: Iterator<Item = MemoryChunk> + Clone {
    fn fmt(&self) -> MemoryMapFmt<Self> {
        MemoryMapFmt { iter: self.clone() }
    }

    fn filter_usable(&self) -> impl Iterator<Item = MemoryChunk> {
        self.clone().filter(|chunk| chunk.is_usable())
    }
}

impl<T> MemoryMap for T where T: Iterator<Item = MemoryChunk> + Clone {}

#[derive(Clone)]
pub struct MemoryMapFmt<I> {
    iter: I,
}


impl<T: Iterator> Iterator for MemoryMapFmt<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: Iterator<Item = MemoryChunk> + Clone> Display for MemoryMapFmt<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let total_bytes_available = self
            .clone()
            .map(|chunk| {
                writeln!(f, "├─ {}", chunk).unwrap();

                if chunk.is_usable() {
                    chunk.length
                } else {
                    0
                }
            })
            .sum::<u64>();

        writeln!(
            f,
            "└─ total memory available: {}",
            types::fmt::Fmt::<u64>::from(total_bytes_available)
        )
    }
}
