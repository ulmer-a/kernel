//! Buddy allocator implementation. This is the same kind of allocator that is used for page frame
//! allocation in the Linux kernel. Note, however, that this allocator requires a global allocator.
//!
//! The code in this file is based on code from the MIT-licensed crate `buddy_system_allocator`:
//!
//! Copyright 2019-2020 Jiajie Chen
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy of this software
//! and associated documentation files (the "Software"), to deal in the Software without
//! restriction, including without limitation the rights to use, copy, modify, merge, publish,
//! distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
//! Software is furnished to do so, subject to the following conditions:
//! The above copyright notice and this permission notice shall be included in all copies or
//! substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
//! BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
//! NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
//! DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#![no_std]
#![feature(btreemap_alloc)]
#![feature(allocator_api)]

extern crate alloc;

use alloc::collections::BTreeSet;
use core::alloc::{Allocator, Layout};

use core::cmp::{max, min};
use core::fmt::{Debug, Formatter};
use core::ops::Range;

/// An allocator using the buddy system.
///
/// The max order of the allocator is specified via the const generic parameter `ORDER`. The frame
/// allocator will only be able to allocate ranges of size up to 2<sup>ORDER</sup>, out of a total
/// range of size at most 2<sup>ORDER + 1</sup> - 1.
pub struct BuddyAllocator<const ORDER: usize = 32, A: Allocator + Clone = alloc::alloc::Global> {
    // buddy system with max order of ORDER
    free_lists: [BTreeSet<usize, A>; ORDER],

    // statistics
    total: usize,
    allocated: usize,
}

impl<const ORDER: usize> BuddyAllocator<ORDER> {
    /// Create an empty (no free blocks) frame allocator. Use `add_range()` to insert free blocks.
    pub fn new() -> Self {
        Self {
            free_lists: core::array::from_fn(|_| BTreeSet::new()),
            total: 0,
            allocated: 0,
        }
    }
}

impl<const ORDER: usize, A: Allocator + Clone> BuddyAllocator<ORDER, A> {
    pub fn new_in(alloc: A) -> Self {
        Self {
            free_lists: core::array::from_fn(|_| BTreeSet::new_in(alloc.clone())),
            total: 0,
            allocated: 0,
        }
    }

    /// Add a free memory range for the allocator to allocate from.
    pub fn add_range(&mut self, range: Range<usize>) {
        // Make sure we don't insert beyond the end of the allocator...
        debug_assert!(
            range.end <= 1 << ORDER,
            "Range end beyond the end of the allocator"
        );
        if range.end > 1 << ORDER {
            return self.add_range(range.start..min(range.end, 1 << ORDER));
        }

        debug_assert!(range.start <= range.end);
        if range.start < range.end {
            // Find the largest possible power of 2 that `range.start` is a multiple of.
            let insertion_alignment = match range.start {
                x if x > 0 => x & (!x + 1),
                x if x == 0 => usize::MAX,
                _ => unreachable!(),
            };

            // Round the range's length down to the next smaller power of two and take into account
            // maximum buddy size of the allocator.
            let max_insertion_size = min(1 << range.len().ilog2(), 1 << (ORDER - 1));
            let actual_insertion_size = min(insertion_alignment, max_insertion_size);

            // Insert start address of the free block into the free list of corresponding order and
            // update statistics accordingly.
            self.total += actual_insertion_size;
            self.free_lists[actual_insertion_size.ilog2() as usize].insert(range.start);

            #[cfg(any(debug_assertions, test))]
            self.assert_block_alignment();

            // If we couldn't insert the whole range in one go, further calls may be required.
            self.add_range((range.start + actual_insertion_size)..range.end);
        }
    }

    /// Allocate a range of frames from the allocator, returning the first frame of the allocated
    /// range.
    pub fn alloc(&mut self, count: usize) -> Option<usize> {
        let size = count.next_power_of_two();
        self.alloc_power_of_two(size)
    }

    /// Allocate a range of frames with the given size and alignment from the allocator, returning
    /// the first frame of the allocated range.
    /// The allocated size is the maximum of the next power of two of the given size and the
    /// alignment.
    pub fn alloc_aligned(&mut self, layout: Layout) -> Option<usize> {
        let size = max(layout.size().next_power_of_two(), layout.align());
        self.alloc_power_of_two(size)
    }

    /// Allocate a range of frames of the given size from the allocator. The size must be a power of
    /// two. The allocated range will have alignment equal to the size.
    fn alloc_power_of_two(&mut self, size: usize) -> Option<usize> {
        let class = size.trailing_zeros() as usize;
        for i in class..self.free_lists.len() {
            // Find the first non-empty size class
            if !self.free_lists[i].is_empty() {
                // Split buffers
                for j in (class + 1..i + 1).rev() {
                    if let Some(block_ref) = self.free_lists[j].iter().next() {
                        let block = *block_ref;
                        self.free_lists[j - 1].insert(block + (1 << (j - 1)));
                        self.free_lists[j - 1].insert(block);
                        self.free_lists[j].remove(&block);
                    } else {
                        return None;
                    }
                }

                let result = self.free_lists[class].iter().next();
                if let Some(result_ref) = result {
                    let result = *result_ref;
                    self.free_lists[class].remove(&result);
                    self.allocated += size;
                    return Some(result);
                } else {
                    return None;
                }
            }
        }
        None
    }

    /// Deallocate a range of frames [frame, frame+count) from the frame allocator.
    ///
    /// The range should be exactly the same when it was allocated, as in heap allocator
    pub fn dealloc(&mut self, start_frame: usize, count: usize) {
        let size = count.next_power_of_two();
        self.dealloc_power_of_two(start_frame, size)
    }

    /// Deallocate a range of frames which was previously allocated by [`alloc_aligned`].
    ///
    /// The layout must be exactly the same as when it was allocated.
    pub fn dealloc_aligned(&mut self, start_frame: usize, layout: Layout) {
        let size = max(layout.size().next_power_of_two(), layout.align());
        self.dealloc_power_of_two(start_frame, size)
    }

    /// Deallocate a range of frames with the given size from the allocator. The size must be a
    /// power of two.
    fn dealloc_power_of_two(&mut self, start_frame: usize, size: usize) {
        let class = size.trailing_zeros() as usize;

        // Merge free buddy lists
        let mut current_ptr = start_frame;
        let mut current_class = class;
        while current_class < self.free_lists.len() {
            let buddy = current_ptr ^ (1 << current_class);
            if self.free_lists[current_class].remove(&buddy) {
                // Free buddy found
                current_ptr = min(current_ptr, buddy);
                current_class += 1;
            } else {
                self.free_lists[current_class].insert(current_ptr);
                break;
            }
        }

        self.allocated -= size;
    }

    /// Check for correct alignment of all blocks in the allocator.
    #[cfg(any(debug_assertions, test))]
    fn assert_block_alignment(&self) {
        for n in (0..ORDER).rev() {
            let size = 1 << n;
            for free_block in &self.free_lists[n] {
                assert_eq!(free_block % size, 0);
            }
        }
    }
}

impl<const ORDER: usize> Debug for BuddyAllocator<ORDER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Free block locations: [ ")?;
        for n in (0..ORDER).rev() {
            if self.free_lists[n].len() > 0 {
                write!(f, "{}: {:?}, ", 2_u32.pow(n as u32), self.free_lists[n])?;
            }
        }
        write!(f, " ]")
    }
}

#[cfg(test)]
mod tests {
    use super::BuddyAllocator;

    #[test]
    fn basic_usage() {
        let mut allocator = BuddyAllocator::<5>::new();
        allocator.add_range(0..32);

        assert_eq!(allocator.alloc(16), Some(0));
        assert_eq!(allocator.alloc(8), Some(16));
        assert_eq!(allocator.alloc(8), Some(24));
    }

    #[test]
    fn full_blocks() {
        let mut allocator = BuddyAllocator::<16>::new();
        allocator.add_range(0..1024);

        assert_eq!(allocator.alloc(512), Some(0));
        assert_eq!(allocator.alloc(512), Some(512));
        assert_eq!(allocator.alloc(1), None);
    }

    #[test]
    fn unaligned() {
        let mut allocator = BuddyAllocator::<16>::new();
        allocator.add_range(0..1025);

        assert_eq!(allocator.alloc(512), Some(0));
        assert_eq!(allocator.alloc(512), Some(512));
        assert_eq!(allocator.alloc(1), Some(1024));
        assert_eq!(allocator.alloc(1), None);
    }
}
