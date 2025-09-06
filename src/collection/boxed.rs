use core::{alloc::Layout, any::type_name, ops::{Deref, DerefMut}, ptr::NonNull};

use crate::alloc::allocator::Allocator;

pub struct KBox<'a, T, A : Allocator> {
    _allocator: &'a A,
    ptr: NonNull<T>
}

impl<'a, T, A : Allocator> KBox<'a, T, A> {
    pub fn new(allocator: &'a A, value: T) -> Self {
        unsafe {
            let layout = Layout::new::<T>();
            let Some(ptr) = allocator.allocate(layout) else {
                panic!("Failed to allocate {} bytes with alignment {} for type '{}'", layout.size(), layout.align(), type_name::<T>());
            };

            let ptr = ptr as *mut T;
            ptr.write(value);

            return Self {
                _allocator: allocator,
                ptr: NonNull::new_unchecked(ptr)
            };
        }
    }

    pub fn into_inner(self) -> T {
        unsafe {
            let value = core::ptr::read(self.ptr.as_ptr());
            core::mem::forget(self);
            return value;
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        return self.ptr.as_ptr();
    }
}

impl<'a, T, A : Allocator> Drop for KBox<'a, T, A> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.as_ptr());
            self._allocator.deallocate(self.as_ptr() as *mut u8);
        }
    }
}

impl<'a, T, A : Allocator> Deref for KBox<'a, T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}

impl<'a, T, A : Allocator> DerefMut for KBox<'a, T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_ptr() }
    }
}
