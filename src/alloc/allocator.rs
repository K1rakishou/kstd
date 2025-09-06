use core::alloc::Layout;


pub trait Allocator {
    fn allocate(&self, layout: Layout) -> Option<*mut u8>;
    fn reallocate(&self, ptr: *mut u8, new_layout: Layout) -> Option<*mut u8>;
    fn deallocate(&self, ptr: *mut u8);
    fn defragment(&self);
}
