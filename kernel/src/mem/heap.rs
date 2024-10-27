use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
static HEAP_MANAGER: HeapManager = HeapManager::new();

struct HeapManager {}

impl HeapManager {
    pub const fn new() -> Self {
        Self {}
    }
}

unsafe impl GlobalAlloc for HeapManager {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!()
    }
}
