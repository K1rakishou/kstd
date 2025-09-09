use core::alloc::Layout;

use crate::alloc::{kallocator::KAllocator, platform};

#[derive(Debug)]
pub struct GlobalAllocator {
    
}

impl GlobalAllocator {
    pub fn new() -> Self {
        return Self {
            
        }
    }
}

impl KAllocator for GlobalAllocator {
    fn allocate(&self, layout: Layout) -> Option<*mut u8> {
        let raw_ptr = if cfg!(unix) {
            platform::linux::allocate(layout)
        } else {
            todo!("Not implemented")
        };

        if raw_ptr.is_null() {
            return None;
        }

        return Some(raw_ptr);
    }

    fn reallocate(&self, ptr: *mut u8, new_layout: Layout) -> Option<*mut u8> {
        let raw_ptr = if cfg!(unix) {
            platform::linux::reallocate(ptr, new_layout)
        } else {
            todo!("Not implemented")
        };

        if raw_ptr.is_null() {
            return None;
        }

        return Some(raw_ptr);
    }

    fn deallocate(&self, ptr: *mut u8) {
        if cfg!(unix) {
            platform::linux::deallocate(ptr)
        } else {
            todo!("Not implemented")
        };
    }

    fn defragment(&self) {
        // no-op
    }
}
