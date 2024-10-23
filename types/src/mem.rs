use core::cmp::{max, min};
use core::fmt::{Display, Formatter, Result};

use crate::fmt::Fmt;

#[derive(Debug, Clone)]
pub struct MemoryChunk {
    pub base_addr: u64,
    pub length: u64,
    pub class: MemoryChunkClass,
}

impl From<multiboot::MemoryRegion> for MemoryChunk {
    fn from(value: multiboot::MemoryRegion) -> Self {
        Self {
            base_addr: value.base_addr,
            length: value.length,
            class: value.kind.into(),
        }
    }
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

impl From<multiboot::MemoryRegionKind> for MemoryChunkClass {
    fn from(value: multiboot::MemoryRegionKind) -> Self {
        use multiboot::MemoryRegionKind;
        match value {
            MemoryRegionKind::Available => Self::Available,
            // ...
            _ => Self::Unusable,
        }
    }
}
