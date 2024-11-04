//! Virtual Memory and Paging

#![allow(unused)]

use core::{marker::PhantomData, pin::Pin};

#[cfg(target_arch = "x86")]
pub mod x86;

pub trait PagingMode: Sized {
    fn create_boot_addr_space();
}

pub struct Unknown;
pub struct Unmapped;
pub struct PageMapped;
pub struct TableMapped;

pub struct PageTableEntry<'table, T, I> {
    inner: Pin<&'table I>,
    _phantom: PhantomData<T>,
}

pub struct GeneralTableEntryMut<'table, T, I> {
    inner: Pin<&'table mut I>,
    _phantom: PhantomData<T>,
}

impl<I: TableEntryImpl> PageTableEntry<'_, Unknown, I> {
    fn is_mapped(&self) -> bool {
        self.inner.is_mapped()
    }

    fn as_table(&self) -> Option<PageTableEntry<'_, TableMapped, I>> {
        todo!()
    }
}

impl<I: TableEntryImpl> GeneralTableEntryMut<'_, Unknown, I> {
    fn as_table_mut(&mut self) -> Option<GeneralTableEntryMut<'_, TableMapped, I>> {
        todo!()
    }
}

pub trait TableEntryImpl {
    fn is_mapped(&self) -> bool;

    fn granularity() -> usize;

    fn map_page(&mut self, ppn: PhysicalPageNumber, user_accessible: bool, writeable: bool);

    fn map_table(&mut self, ppn: PhysicalPageNumber);

    fn unmap(&mut self) -> Option<PhysicalPageNumber>;
}

/// Identifies a single page frame within the physical memory space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalPageNumber {
    ppn: usize,
}

impl PhysicalPageNumber {
    pub fn into_physical_ptr<T>(self) -> *mut T {
        (self.ppn * 4096) as *mut T
    }
}

/// Identifies a single virtual memory page in a virtual address space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualPageNumber {
    vpn: usize,
}

impl VirtualPageNumber {
    pub fn from_addr(addr: u64) -> Self {
        Self {
            vpn: (addr / 4096) as usize,
        }
    }
}
