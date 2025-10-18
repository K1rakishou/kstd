use core::{alloc::Layout, any::type_name, ops::{Deref, DerefMut}, ptr::NonNull};
use crate::alloc::kallocator::KAllocator;

pub struct KBox<'allocator, T, A : KAllocator> {
    _allocator: &'allocator A,
    ptr: NonNull<T>
}

#[allow(dead_code)]
impl<'allocator, T, A : KAllocator> KBox<'allocator, T, A> {
    pub fn new(allocator: &'allocator A, value: T) -> Self {
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

    pub fn as_ptr(&self) -> *mut T {
        return self.ptr.as_ptr();
    }
}

impl<'allocator, T, A : KAllocator> Drop for KBox<'allocator, T, A> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.as_ptr());

            let layout = Layout::new::<T>();
            self._allocator.deallocate(self.as_ptr() as *mut u8, layout);
        }
    }
}

impl<'allocator, T, A : KAllocator> Deref for KBox<'allocator, T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}

impl<'allocator, T, A : KAllocator> DerefMut for KBox<'allocator, T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_ptr() }
    }
}

#[allow(unused_imports)]
mod test {
    use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
    use crate::alloc::{kglobal_allocator::KGlobalAllocator, kbox::KBox};

    #[test]
    fn test_kbox_drop_is_called() {
        let allocator = KGlobalAllocator::new();
        let dropflag = Arc::new(AtomicBool::new(false));
        struct MustBeDropped(Arc<AtomicBool>);
        
        impl Drop for MustBeDropped {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }

        {
            let dropflag = Arc::clone(&dropflag);
            KBox::new(&allocator, MustBeDropped(dropflag));
        }

        assert_eq!(true, dropflag.load(Ordering::Relaxed));
    }
}
