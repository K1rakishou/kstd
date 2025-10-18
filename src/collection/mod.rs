use std::alloc::Layout;

pub mod kvec;
pub mod kheap;
pub mod khashmap;
pub mod krbtree;
pub mod kring_buffer;

pub fn layout_from_capacity<T>(capacity: usize) -> Layout {
    let elem_size = core::mem::size_of::<T>();
    let elem_align = core::mem::align_of::<T>();

    assert!(capacity > 0, "Capacity must be > 0");
    assert!(elem_size > 0, "ZSTs are not supported yet!");

    let alloc_size = capacity
        .checked_mul(elem_size)
        .expect("Capacity calculation overflow");

    return Layout::from_size_align(alloc_size, elem_align).unwrap();
}
