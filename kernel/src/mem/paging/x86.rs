//! Paging/virtual memory implementation for IA-32 architecture (x86).

use super::{PhysicalPageNumber, TableEntryImpl};

pub struct Paging;

type PageDirectoryEntry = GeneralTableEntry<2>;

type PageTableEntry = GeneralTableEntry<1>;

/// A Page Directory or Page Table entry.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq)]
pub struct GeneralTableEntry<const LEVEL: usize> {
    inner: u32,
}

impl<const LEVEL: usize> GeneralTableEntry<LEVEL> {
    /// Whether the entry is present (a page table or a physical page is mapped).
    fn is_present(self) -> bool {
        self.inner & 1 != 0
    }
}

impl TableEntryImpl for GeneralTableEntry<2> {
    fn is_mapped(&self) -> bool {
        self.is_present()
    }

    fn map_page(&mut self, ppn: PhysicalPageNumber, user_accessible: bool, writeable: bool) {
        self.inner = EntryBuilder::default()
            .with_huge_page(ppn)
            .with_options(writeable, user_accessible)
            .build_present();
    }

    fn map_table(&mut self, ppn: PhysicalPageNumber) {
        self.inner = EntryBuilder::default()
            .with_ppn(ppn)
            .with_options(true, true)
            .build_present();
    }

    fn unmap(&mut self) -> Option<PhysicalPageNumber> {
        todo!()
    }

    fn granularity() -> usize {
        // granularity of PageDirectory is 4 MiB (1024 pages)
        4096 * 1024
    }
}

impl TableEntryImpl for GeneralTableEntry<1> {
    fn is_mapped(&self) -> bool {
        self.is_present()
    }

    fn map_page(&mut self, ppn: PhysicalPageNumber, user_accessible: bool, writeable: bool) {
        self.inner = EntryBuilder::default()
            .with_ppn(ppn)
            .with_options(writeable, user_accessible)
            .build_present();
    }

    fn map_table(&mut self, _ppn: PhysicalPageNumber) {
        todo!() // Error
    }

    fn unmap(&mut self) -> Option<PhysicalPageNumber> {
        todo!()
    }

    fn granularity() -> usize {
        // Granularity of PageTable is the page size
        4096
    }
}

#[derive(Debug, Default)]
pub struct EntryBuilder {
    inner: u32,
}

impl EntryBuilder {
    #[expect(clippy::cast_possible_truncation, reason = "Range is asserted")]
    fn with_ppn(mut self, ppn: PhysicalPageNumber) -> Self {
        assert!(ppn.ppn < 2usize.pow(20), "PPN may only use 20 bit max.");
        self.inner |= (ppn.ppn << 12) as u32;
        self
    }

    fn with_options(mut self, writeable: bool, user_accessible: bool) -> Self {
        if writeable {
            self.inner |= 1 << 1;
        }

        if user_accessible {
            self.inner |= 1 << 2;
        }

        self
    }

    fn with_huge_page(mut self, _ppn: PhysicalPageNumber) -> Self {
        self.inner |= 1 << 7;
        todo!()
    }

    fn build_present(self) -> u32 {
        self.inner | 1
    }
}
