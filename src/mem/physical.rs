use core::cmp::{max, min};
use core::fmt::{Display, Formatter, Result};

use crate::mem::fmt::Fmt;

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

#[derive(Debug, Clone)]
pub struct MemoryChunk {
    pub base_addr: u64,
    pub length: u64,
    pub class: MemoryChunkClass,
}

impl MemoryChunk {
    pub fn crop_start(self, min_addr: u64) -> Option<Self> {
        if min_addr < self.end_addr() {
            Some(Self {
                base_addr: min_addr,
                length: self.end_addr() - max(self.base_addr, min_addr),
                ..self
            })
        } else {
            None
        }
    }

    pub fn crop_end(self, max_addr: u64) -> Option<Self> {
        if max_addr > self.base_addr {
            Some(MemoryChunk {
                length: if max_addr < self.end_addr() {
                    min(self.end_addr(), max_addr) - self.base_addr
                } else {
                    self.length
                },
                ..self
            })
        } else {
            None
        }
    }

    pub fn crop(self, min_addr: u64, max_addr: u64) -> Option<Self> {
        self.crop_start(min_addr)
            .and_then(|chunk| chunk.crop_end(max_addr))
    }

    pub fn end_addr(&self) -> u64 {
        self.base_addr + self.length
    }

    pub fn first_page(&self) -> usize {
        (self.base_addr / 4096) as usize
    }

    pub fn last_page(&self) -> usize {
        ((self.base_addr + self.length) / 4096) as usize
    }

    pub fn page_count(&self) -> usize {
        self.last_page() - self.first_page() + 1
    }

    pub fn is_usable(&self) -> bool {
        self.class == MemoryChunkClass::Available
    }
}

impl core::fmt::Display for MemoryChunk {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "@ 0x{:x}: {} ({})",
            self.base_addr,
            Fmt::<u64>::from(self.length),
            self.class
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryChunkClass {
    Available,
    Unusable,
    Reclaimable,
}

impl Display for MemoryChunkClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            MemoryChunkClass::Available => "usable",
            MemoryChunkClass::Unusable => "reserved",
            MemoryChunkClass::Reclaimable => "reclaimable",
        })
    }
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
            Fmt::<u64>::from(total_bytes_available)
        )
    }
}
