use std::{alloc::{alloc, dealloc, Layout}, ops::{Deref, DerefMut}, ptr::NonNull};

#[derive(Debug)]
pub struct KBox<T> {
    ptr: NonNull<T>
}

impl<T> KBox<T> {
    pub fn new(value: T) -> Self {
        unsafe {
            let layout = Layout::new::<T>();
            let ptr = alloc(layout) as *mut T;

            ptr.write(value);

            return Self {
                ptr: NonNull::new_unchecked(ptr)
            };
        }
    }

    pub fn into_inner(self) -> T {
        unsafe {
            let value = self.ptr.read();
            std::mem::forget(self);
            return value;
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        return self.ptr.as_ptr();
    }
}

impl<T> Drop for KBox<T> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(self.as_ptr());

            let layout = Layout::new::<T>();
            dealloc(self.as_ptr() as *mut u8, layout);
        }
    }
}

impl<T> Deref for KBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}

impl<T> DerefMut for KBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_ptr() }
    }
}
