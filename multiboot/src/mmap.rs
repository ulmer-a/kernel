//! Iterator and structs to traverse and represent entries of the Multiboot memory map.

/// Provides an iterator over the multiboot memory map. The `'mmap` lifetime parameter describes
/// the lifetime of the underlying memory buffer containing the memory map.
#[derive(Clone)]
pub struct MemoryMapIter<'mmap> {
    /// Reference to the buffer from [`BootInfo`] that contains all the memory map entries.
    buffer: &'mmap [u8],
}

impl<'mmap> From<&'mmap [u8]> for MemoryMapIter<'mmap> {
    fn from(buffer: &'mmap [u8]) -> Self {
        Self { buffer }
    }
}

impl Iterator for MemoryMapIter<'_> {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: We want to take a reference to the first memory map entry that is contained
        // in the buffer. So, if the buffer is large enough, transmute the first matching bytes
        // to a MemoryMapEntry. This type has the correct layout and uses repr(C).
        let (head, body, _) = unsafe { self.buffer.align_to::<MemoryMapEntry>() };
        assert_eq!(head.len(), 0);

        let entry = body.first()?;
        self.buffer = &self.buffer[entry.offset_to_next()..];
        Some(entry.into())
    }
}

impl core::fmt::Debug for MemoryMapIter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A contiguous region of physical memory reported by the Multiboot memory map.
#[derive(Clone, Copy, PartialEq)]
pub struct MemoryRegion {
    /// The start address of the memory region.
    pub base_addr: u64,
    /// The length of the memory region in bytes.
    pub length: u64,
    /// The type of the memory region.
    pub kind: MemoryRegionKind,
}

impl From<&MemoryMapEntry> for MemoryRegion {
    fn from(entry: &MemoryMapEntry) -> Self {
        Self {
            base_addr: entry.base_addr,
            length: entry.length,
            kind: match entry.r#type {
                1 => MemoryRegionKind::Available,
                3 => MemoryRegionKind::Acpi,
                4 => MemoryRegionKind::Reserved,
                5 => MemoryRegionKind::Defective,
                _ => MemoryRegionKind::Unknown,
            },
        }
    }
}

impl core::fmt::Debug for MemoryRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:?} @ {:?} ({:?} KiB)",
            self.kind,
            self.base_addr as *const u8,
            self.length >> 10,
        )
    }
}

/// Describes the availability of the memory referenced by a [`MemoryRegion`] as reported by the
/// `type` field of a Multiboot memory map entry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionKind {
    Available,
    Acpi,
    Reserved,
    Defective,
    Unknown,
}

/// Represents an entry in the multiboot memory map. The buffer consists of one or more of the
/// following size/structure pairs (`size` is really just used for skipping to the next pair):
///
/// ```text
///         +-------------------+  <-- Start of struct MemoryMapEntry
/// -4      | size              |
///         +-------------------+  <-- `size` is from here to next entry
/// 0       | base_addr         |
/// 8       | length            |
/// 16      | type              |
///         +-------------------+  <-- End of struct MemoryMapEntry
/// ```
///
/// where `size` is the size of the associated structure in bytes, which can be greater than the
/// minimum of 20 bytes. `base_addr` is the starting address. `length` is the size of the memory
/// region in bytes.
#[repr(C)]
struct MemoryMapEntry {
    /// When 4 is added to `size`, the result can be used as an offset to skip to the next memory
    /// map entry in the mmap buffer. According to the specification, this offset can be larger
    /// than than the size of this structure (20 bytes).
    size: u32,

    /// The start address of the memory region described by this entry.
    base_addr: u64,

    /// The size in bytes of the memory region described by this entry.
    length: u64,

    /// The type of the memory described by this entry.
    ///
    /// * 1: "available RAM"
    /// * 3: "usable memory holding ACPI information"
    /// * 4: "reserved memory which needs to be preserved on hibernation"
    /// * 5: "memory which is occupied by defective RAM modules"
    r#type: u32,
}

impl MemoryMapEntry {
    /// Returns the offset from the start address of this memory map entry to the next entry in the
    /// buffer.
    pub fn offset_to_next(&self) -> usize {
        self.size as usize + 4
    }
}
