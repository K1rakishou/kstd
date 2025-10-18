use core::alloc::{Layout};
use crate::alloc::{kallocator::KAllocator};

#[derive(Debug)]
pub struct KGlobalAllocator {
    
}

#[allow(dead_code)]
impl KGlobalAllocator {
    pub fn new() -> Self {
        return Self {
            
        }
    }
}

impl KAllocator for KGlobalAllocator {
    fn allocate(&self, layout: Layout) -> Option<*mut u8> {
        let raw_ptr = unsafe { std::alloc::alloc(layout) };
        if raw_ptr.is_null() {
            return None;
        }

        return Some(raw_ptr);
    }

    fn reallocate(&self, ptr: *mut u8, old_layout: Layout, new_size: usize) -> Option<*mut u8> {
        let raw_ptr = unsafe { std::alloc::realloc(ptr, old_layout, new_size) };
        if raw_ptr.is_null() {
            return None;
        }

        return Some(raw_ptr);
    }

    fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        unsafe { std::alloc::dealloc(ptr, layout); }
    }

    fn defragment(&self) {
        // no-op
    }
}
