//! Virtual Memory and Paging

#[cfg(target_arch = "x86")]
pub mod x86;

pub trait PageFrameAlloc {
    fn alloc_page(&mut self) -> PhysicalPageNumber;
}

pub trait PagingMode {
    fn create_boot_mappings(allocator: &mut dyn PageFrameAlloc) -> impl AddressSpace;
}

/// A virtual memory address space.
pub trait AddressSpace {
    // ...

    /// Switch to this address space on the current CPU.
    fn load(&self);

    // /// Call this as soon as the address space has been unloaded to release locks.
    // fn unloaded();
}

pub struct UserAddressSpace {
    // ...
}

impl AddressSpace for UserAddressSpace {
    fn load(&self) {
        todo!()
    }
}

// pub trait PageTable: core::ops::Index<VirtualPageNumber, Output = dyn PageTableEntry> {
//     type Entry: PageTableEntry;

//     fn granularity() -> usize;

//     fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn PageTableEntry>;
// }

// pub trait PageTableEntry {
//     fn map_page(&mut self, ppn: ()) -> Result<(), ()>;

//     fn unmap(&mut self) -> Result<PhysicalPageMut, ()> {
//         Err(())
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalPageNumber {
    ppn: usize,
}

// impl PhysicalPageMut {
//     pub unsafe fn get_physical_ptr<T>(&self) -> *mut T {
//         // (ppn * PAGE_SIZE) as *mut T
//         todo!()
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualPageNumber {
    vpn: usize,
}
