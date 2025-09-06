use core::{alloc::Layout, ffi::c_void};

pub fn allocate(layout: Layout) -> *mut u8 {
    unsafe { libc::aligned_alloc(layout.align(), layout.size()) as *mut u8 }
}

pub fn reallocate(ptr: *mut u8, new_layout: Layout) -> *mut u8 {
    unsafe { libc::realloc(ptr as *mut c_void, new_layout.size()) as *mut u8 }
}

pub fn deallocate(ptr: *mut u8) {
    unsafe { libc::free(ptr as *mut c_void) }
}
