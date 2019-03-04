mod linked_list;
mod util;

#[path = "bump.rs"]
mod imp;

#[cfg(test)]
mod tests;

use mutex::Mutex;
use alloc::alloc::{GlobalAlloc, AllocErr, Layout};
use std::cmp::max;

use pi::atags;

/// Thread-safe (locking) wrapper around a particular memory allocator.
#[derive(Debug)]
pub struct Allocator(Mutex<Option<imp::Allocator>>);

impl Allocator {
    /// Returns an uninitialized `Allocator`.
    ///
    /// The allocator must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        Allocator(Mutex::new(None))
    }

    /// Initializes the memory allocator.
    ///
    /// # Panics
    ///
    /// Panics if the system's memory map could not be retrieved.
    pub fn initialize(&self) {
        let (start, end) = memory_map().expect("failed to find memory map");
        *self.0.lock() = Some(imp::Allocator::new(start, end));
    }
}

unsafe impl GlobalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().as_mut().expect("allocator uninitialized").alloc(layout).unwrap()
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().as_mut().expect("allocator uninitialized").dealloc(ptr, layout);
    }
}

extern "C" {
    static _end: u8;
}

/// Returns the (start address, end address) of the available memory on this
/// system if it can be determined. If it cannot, `None` is returned.
///
/// This function is expected to return `Some` under all normal cirumstances.
fn memory_map() -> Option<(usize, usize)> {
    let binary_end = unsafe { (&_end as *const u8) as u32 };

    let mut start = binary_end as usize;
    let mut len = None;
    for a in atags::Atags::get() {
        if a.mem().is_some() {
            let mem = a.mem().unwrap();
            if mem.start as usize > start {
                start = mem.start as usize;
                len = Some(mem.size as usize);
            } else {
                len = Some(mem.size as usize - (binary_end - mem.start) as usize);
            }
            break;
        }
    }

    match len {
        Some(l) => Some((start, l + start)),
        None => None
    }
}
