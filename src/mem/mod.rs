#[derive(Debug, Clone)]
pub struct MemoryChunk {
    pub base_addr: u64,
    pub length: u64,
    pub kind: MemoryChunkClass,
}

impl MemoryChunk {
    pub fn is_usable(&self) -> bool {
        self.kind == MemoryChunkClass::Available
    }
}

impl core::fmt::Display for MemoryChunk {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "@ 0x{:x} ({})", self.base_addr, self.kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryChunkClass {
    Available,
    Unusable,
    Reclaimable,
}

impl core::fmt::Display for MemoryChunkClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            MemoryChunkClass::Available => "usable",
            MemoryChunkClass::Unusable => "reserved",
            MemoryChunkClass::Reclaimable => "reclaimable",
        })
    }
}
