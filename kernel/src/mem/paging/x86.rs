//! Paging/virtual memory implementation for IA-32 architecture (x86).

use super::PageFrameAlloc;

pub struct BootIdentMapping {}

impl BootIdentMapping {
    pub fn new(
        allocator: &mut dyn PageFrameAlloc,
        mappings: impl Iterator<Item = types::mem::MemoryChunk>,
    ) -> Self {
        let mut builder = BootIdentMappingBuilder::new(allocator);
        for mapping in mappings {
            builder.add_mapping(mapping);
        }
        Self {}
    }
}

pub struct BootIdentMappingBuilder<'alloc> {
    allocator: &'alloc mut dyn PageFrameAlloc,
    page_directory: *mut (),
}

impl<'alloc> BootIdentMappingBuilder<'alloc> {
    fn new(allocator: &'alloc mut dyn PageFrameAlloc) -> Self {
        let page_directory = allocator.alloc_page();
        Self {
            allocator,
            page_directory: unsafe { todo!() },
        }
    }

    fn add_mapping(&mut self, region: types::mem::MemoryChunk) {
        let page_dir = unsafe { self.page_directory.as_mut().unwrap() };
        todo!()
    }
}

// type PageDirectory = PageTable32<2>;

// type PageTable = PageTable32<1>;

// #[repr(C, align(4096))]
// struct PageTable32<const LEVEL: usize> {
//     tables: [u32; 1024],
// }

// struct PageTableIterMut<'table, T> {
//     entries: &'table mut [u32],
// }

// impl<'table, T: super::PageTable> Iterator for PageTableIterMut<'table, T> {
//     type Item = &'table mut dyn super::PageTableEntry;

//     fn next(&mut self) -> Option<Self::Item> {
//         let entry = &mut self.entries.get(0)?;
//         self.entries = &mut self.entries[1..];
//         Some(unsafe { core::mem::transmute(entry) })
//     }
// }

// impl super::PageTable for PageDirectory {
//     fn granularity() -> usize {
//         4096 * 1024
//     }

//     fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn super::PageTableEntry> {
//         todo!()
//     }
// }

// impl super::PageTable for PageTable {
//     fn granularity() -> usize {
//         4096
//     }

//     fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn super::PageTableEntry> {
//         todo!()
//     }
// }

// struct PageTableEntry32<T> {
//     inner: u32,
//     _phantom: core::marker::PhantomData<T>,
// }

// impl<T> From<u32> for PageTableEntry32<T> {
//     fn from(value: u32) -> Self {
//         Self {
//             inner: value,
//             _phantom: core::marker::PhantomData,
//         }
//     }
// }

// impl super::PageTableEntry for PageTableEntry32<PageDirectory> {
//     fn map_page(&mut self, _ppn: ()) -> Result<(), ()> {
//         todo!()
//     }
// }

// impl super::PageTableEntry for PageTableEntry32<PageTable> {
//     fn map_page(&mut self, _ppn: ()) -> Result<(), ()> {
//         todo!()
//     }
// }
