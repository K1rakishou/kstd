use core::alloc::Layout;

pub trait KAllocator {
    fn allocate(&self, layout: Layout) -> Option<*mut u8>;
    fn reallocate(&self, ptr: *mut u8, old_layout: Layout, new_size: usize) -> Option<*mut u8>;
    fn deallocate(&self, ptr: *mut u8, layout: Layout);
    #[allow(dead_code)]
    fn defragment(&self);
}
