//! Virtual Memory and Paging
//!
//! We need following features
//! * Identity map areas of physical memory to virtual memory (kernel addr space)
//! * Map single page for user requests
//! * ...

#[cfg(target_arch = "x86")]
mod x86;

// trait PageFrameAllocator
// struct VirtualPage
// struct Physical Page
// MapOptions
