use core::{alloc::Layout, ops::{Deref, DerefMut, Index, IndexMut}, ptr::NonNull, slice};
use core::fmt::Debug;
use std::cmp::Ordering;

use crate::{alloc::kallocator::KAllocator, collection::layout_from_capacity};

pub struct KVec<'allocator, T, A : KAllocator> {
    _allocator: &'allocator A,
    _buffer: NonNull<T>,
    _capacity: usize,
    _length: usize
}

#[allow(dead_code)]
impl<'allocator, T, A : KAllocator> KVec<'allocator, T, A> {
    pub fn new(allocator: &'allocator A) -> Self {
        return Self {
            _allocator: allocator,
            _buffer: NonNull::dangling(),
            _capacity: 0,
            _length: 0
        }
    }

    pub fn with_capacity(allocator: &'allocator A, capacity: usize) -> Self {
        let (new_buffer, new_capacity) = Self::grow(allocator, NonNull::dangling(), capacity, 0, 0);
        let this = Self {
            _allocator: allocator,
            _buffer: new_buffer,
            _capacity: new_capacity,
            _length: 0
        };

        return this;
    }

    pub fn from_slice(allocator: &'allocator A, slice: &[T]) -> Self {
        let (new_buffer, new_capacity) = Self::grow(allocator, NonNull::dangling(), slice.len(), 0, 0);
        let mut this = Self {
            _allocator: allocator,
            _buffer: new_buffer,
            _capacity: new_capacity,
            _length: 0
        };

        this.extend_from_slice(slice);

        return this;
    }

    pub fn push(&mut self, value: T) {
        if self._capacity <= self._length {
            let (new_buffer, new_capacity) = Self::grow(self._allocator, self._buffer, self._capacity, 0, self._length);

            self._buffer = new_buffer;
            self._capacity = new_capacity;
        }

        let index = self._length;

        unsafe {
            self._buffer
                .offset(index as isize)
                .write(value);
        }

        self._length = index + 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        
        let index = self._length - 1;

        let value = unsafe {
            let element_ptr = self._buffer
                .offset(index as isize)
                .as_ptr();
            
            core::ptr::read(element_ptr)
        };

        self._length = index;

        return Some(value);
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self._length || self._length == 0 {
            return None;
        }

        if index == self._length - 1 {
            return self.pop();
        }

        let value = unsafe {
            let element_ptr = self._buffer
                .offset(index as isize)
                .as_ptr();
            
            core::ptr::read(element_ptr)
        };

        let src = unsafe {
            self._buffer
                .offset((index as isize) + 1)
                .as_ptr()
        };

        let dst = unsafe {
            self._buffer
                .offset(index as isize)
                .as_ptr()
        };
        
        let count = self._length - index;
        unsafe { core::ptr::copy(src, dst, count) };

        self._length = self._length - 1;
        return Some(value);
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self._length {
            return None;
        }

        let value = unsafe {
            &*self._buffer
                .offset(index as isize)
                .as_ptr()
        };

        return Some(value);
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self._length {
            return None;
        }

        let value = unsafe {
            &mut *self._buffer
                .offset(index as isize)
                .as_ptr()
        };

        return Some(value);
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_ref().iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.as_mut().iter_mut()
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self._length;
    }

    #[inline]
    pub fn cap(&self) -> usize {
        return self._capacity;
    }

    #[inline]
    pub fn last_index(&self) -> Option<usize> {
        return self._length.checked_sub(1);
    }

    #[inline]
    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        
        let last_element = unsafe { self._buffer.add(self._length).as_ref() };
        return Some(last_element);
    }

    pub fn reserve(&mut self, additional: usize) {
        let Some(free_space_length) = self._capacity.checked_sub(self._length) else {
            panic!("Subtraction overflow!");
        };
            
        if additional > free_space_length {
            let (new_buffer, new_capacity) = Self::grow(self._allocator, self._buffer, self._capacity, additional, self._length);
            self._buffer = new_buffer;
            self._capacity = new_capacity;
        }
    }

    pub fn extend_from_slice(&mut self, slice: &[T]) {
        let slice_len = slice.len();
        let available_len = self._capacity as isize - self._length as isize;

        if available_len < slice_len as isize {
            let (new_buffer, new_capacity) = Self::grow(self._allocator, self._buffer, self._capacity, slice_len, self._length);
            self._buffer = new_buffer;
            self._capacity = new_capacity;
        }

        unsafe {
            let src_ptr = slice.as_ptr();
            let dst_ptr = self._buffer.add(self._length).as_ptr();
            core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, slice_len);

            self._length += slice_len;
        };
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        return self._length == 0;
    }

    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        return unsafe { slice::from_raw_parts(self.as_ptr(), self._length) };
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        return self._buffer.as_ptr() as *const T;
    }

    fn grow(allocator: &'allocator A, buffer: NonNull<T>, old_capacity: usize, additional: usize, length: usize) -> (NonNull<T>, usize) {
        let elem_size = core::mem::size_of::<T>();
        let elem_align = core::mem::align_of::<T>();

        if elem_size == 0 {
            panic!("ZSTs are not supported yet!");
        }

        let total_capacity = old_capacity + additional;
        
        let new_capacity = if (length == 0 && total_capacity > 0) || additional > 0 {
            total_capacity
        } else {
            match total_capacity {
                0 => 4,
                cap => {
                    if cap == usize::MAX {
                        panic!("Capacity overflow");
                    }

                    cap.saturating_mul(2)
                },
            }
        };

        let new_buffer = {
            let new_size = new_capacity
                .checked_mul(elem_size)
                .expect("New size calculation overflow");

            unsafe {
                let ptr_raw = if length == 0 {
                    // length == 0 is the very first allocation. At this point buffer pointer is dangling so we need to allocate it.
                    let new_layout = Layout::from_size_align(new_size, elem_align).unwrap();
                    let new_ptr = allocator.allocate(new_layout);

                    new_ptr
                } else {
                    // length != 0 means that we are reallocating which means it's safe to access buffer's pointer.
                    let old_size = old_capacity
                        .checked_mul(elem_size)
                        .expect("Old size calculation overflow");

                    let old_layout = Layout::from_size_align(old_size, elem_align).unwrap();
                    let old_ptr = buffer.as_ptr() as *mut u8;
                    let new_ptr = allocator.reallocate(old_ptr, old_layout, new_size);

                    new_ptr
                };

                let Some(ptr_raw) = ptr_raw else {
                    todo!("Try to defragment the memory in the allocator, or something");
                };

                NonNull::new_unchecked(ptr_raw as *mut T)
            }
        };

        assert!(new_capacity > length, "Capacity must always be greater than length");
        return (new_buffer, new_capacity);
    }
}

impl<'allocator, T : Clone, A : KAllocator> KVec<'allocator, T, A> {
    pub fn resize(&mut self, new_len: usize, value: T) {
        if new_len <= self.len() {
            return;
        }

        let additional = new_len - self.len();
        self.reserve(additional);

        for _ in 0..additional - 1 {
            let index = self._capacity;

            unsafe {
                self._buffer
                    .offset(index as isize)
                    .write(value.clone());
            }

            self._length = index + 1;
        }

        unsafe {
            self._buffer
                .offset(additional as isize)
                .write(value);
        }
    }
}

impl<'allocator, T, A : KAllocator> Drop for KVec<'allocator, T, A> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>() {
            for offset in 0 .. self._length {
                unsafe { 
                    let element_ptr = self._buffer.add(offset).as_ptr();
                    core::ptr::drop_in_place(element_ptr);
                }
            }
        }

        if self._capacity > 0 {
            let layout = layout_from_capacity::<T>(self._capacity);
            self._allocator.deallocate(self._buffer.as_ptr() as *mut u8, layout);
        }

        self._length = 0;
        self._capacity = 0;
    }
}

impl<'allocator, T, A : KAllocator> Index<usize> for KVec<'allocator, T, A> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<'allocator, T, A : KAllocator> IndexMut<usize> for KVec<'allocator, T, A> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<'allocator, T, A : KAllocator> Debug for KVec<'allocator, T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "KVec(")?;
        write!(f, "buffer: 0x{:p}, ", self._buffer.as_ptr())?;
        write!(f, "capacity: {}, ", self._capacity)?;
        write!(f, "length: {}", self._length)?;
        write!(f, ")")?;
        return Ok(());
    }
}

impl<'allocator, T, A : KAllocator> Deref for KVec<'allocator, T, A> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        return unsafe { core::slice::from_raw_parts(self._buffer.as_ptr(), self._length) };
    }
}

impl<'allocator, T, A : KAllocator> DerefMut for KVec<'allocator, T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return unsafe { core::slice::from_raw_parts_mut(self._buffer.as_ptr(), self._length) };
    }
}

impl<'allocator, T, A : KAllocator> AsRef<[T]> for KVec<'allocator, T, A> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<'allocator, T, A : KAllocator> AsMut<[T]> for KVec<'allocator, T, A> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'allocator, T, A : KAllocator> IntoIterator for KVec<'allocator, T, A> {
    type Item = T;
    type IntoIter = KVecIterator<'allocator, T, A>;

    fn into_iter(self) -> Self::IntoIter {
        return KVecIterator::new(self);
    }
}

pub struct KVecIterator<'allocator, T, A : KAllocator> {
    _kvec: KVec<'allocator, T, A>,
    _index: usize
}

impl<'allocator, T, A : KAllocator> KVecIterator<'allocator, T, A> {
    pub fn new(kvec: KVec<'allocator, T, A>) -> Self {
        return Self {
            _kvec: kvec,
            _index: 0
        };
    }
}

impl<'allocator, T, A : KAllocator> Iterator for KVecIterator<'allocator, T, A> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self._index;
        if index >= self._kvec._length {
            return None;
        }
        
        let element = unsafe {
            let ptr = self._kvec._buffer
                .offset(self._index as isize)
                .as_ptr();

            core::ptr::read(ptr)
        };
        
        self._index += 1;
        return Some(element);
    }
}

impl<'allocator, T : PartialEq, A : KAllocator> PartialEq for KVec<'allocator, T, A> {
    fn eq(&self, other: &Self) -> bool {
        if !self.len().eq(&other.len()) {
            return false;
        }

        let length = self.len();

        for idx in 0 .. length {
            let elem1 = self.index(idx);
            let elem2 = other.index(idx);

            if !elem1.eq(elem2) {
                return false;
            }
        }

        return true;
    }
}

impl<'allocator, T : PartialOrd, A : KAllocator> PartialOrd for KVec<'allocator, T, A> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let Some(length_cmp) = self.len().partial_cmp(&other.len()) else {
            return None;
        };
        
        if length_cmp != Ordering::Equal {
            return Some(length_cmp);
        }

        let length = self.len();

        for idx in 0 .. length {
            let elem1 = self.index(idx);
            let elem2 = other.index(idx);

            let Some(elem_cmp) = elem1.partial_cmp(&elem2) else {
                return None;
            };
        
            if elem_cmp != Ordering::Equal {
                return Some(elem_cmp);
            }
        }

        return Some(Ordering::Equal);
    }
}

impl<'allocator, T : Clone, A : KAllocator> Clone for KVec<'allocator, T, A> {
    fn clone(&self) -> Self {
        let mut cloned = KVec::with_capacity(self._allocator, self._capacity);
        cloned.extend_from_slice(&self);
        return cloned;
    }
}

#[allow(unused_imports)]
mod test {
    use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

    use crate::alloc::kglobal_allocator::KGlobalAllocator;
    use super::KVec;

    #[test]
    fn test_kvec() {
        let allocator = KGlobalAllocator::new();
        let mut kvec = KVec::<u64, KGlobalAllocator>::new(&allocator);

        for i in 0..1024 {
            kvec.push(i);
        }

        assert_eq!(1024, kvec.len());

        for i in 0..1024 {
            assert_eq!(i, kvec[i] as usize);
            assert_eq!(i, *kvec.get_mut(i).unwrap() as usize);
        }

        for i in (0..1024).rev() {
            assert_eq!(i, kvec.pop().unwrap());
        }

        assert_eq!(0, kvec.len());
        assert_eq!(None, kvec.pop());
    }

    #[test]
    fn test_kvec_deref() {
        let allocator = KGlobalAllocator::new();

        fn accepts_slice(slice: &[usize]) {
            let _ = slice;
        }

        fn accepts_slice_mut(slice_mut: &mut[usize]) {
            let _ = slice_mut;
        }

        let mut kvec = KVec::<usize, KGlobalAllocator>::new(&allocator);
        kvec.push(11223344);

        accepts_slice(&kvec);
        accepts_slice_mut(&mut kvec);
    }

    #[test]
    fn test_kvec_iter() {
        let allocator = KGlobalAllocator::new();
        let mut kvec = KVec::<usize, KGlobalAllocator>::new(&allocator);
        kvec.push(1);
        kvec.push(2);
        kvec.push(3);
        kvec.push(4);
        kvec.push(5);
        kvec.push(6);

        let mut iter = kvec.iter_mut();
        assert_eq!(1, *iter.next().unwrap());
        assert_eq!(2, *iter.next().unwrap());
        assert_eq!(3, *iter.next().unwrap());
        assert_eq!(4, *iter.next().unwrap());
        assert_eq!(5, *iter.next().unwrap());
        assert_eq!(6, *iter.next().unwrap());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_kvec_into_iter() {
        let allocator = KGlobalAllocator::new();
        let mut kvec = KVec::<usize, KGlobalAllocator>::new(&allocator);
        kvec.push(1);
        kvec.push(2);
        kvec.push(3);
        kvec.push(4);
        kvec.push(5);
        kvec.push(6);
        
        let mut iter = kvec.into_iter();
        assert_eq!(1, iter.next().unwrap());
        assert_eq!(2, iter.next().unwrap());
        assert_eq!(3, iter.next().unwrap());
        assert_eq!(4, iter.next().unwrap());
        assert_eq!(5, iter.next().unwrap());
        assert_eq!(6, iter.next().unwrap());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_kvec_with_initial_capacity() {
        let allocator = KGlobalAllocator::new();

        let mut kvec = KVec::<usize, KGlobalAllocator>::with_capacity(&allocator, 4);
        assert_eq!(4, kvec._capacity);
        assert_eq!(0, kvec._length);
        
        kvec.push(1);
        kvec.push(2);
        kvec.push(3);
        kvec.push(4);
        assert_eq!(4, kvec._capacity);

        kvec.push(5);
        assert_eq!(8, kvec._capacity);
        
        kvec.push(6);
        
        let mut iter = kvec.into_iter();
        assert_eq!(1, iter.next().unwrap());
        assert_eq!(2, iter.next().unwrap());
        assert_eq!(3, iter.next().unwrap());
        assert_eq!(4, iter.next().unwrap());
        assert_eq!(5, iter.next().unwrap());
        assert_eq!(6, iter.next().unwrap());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_kvec_with_0_initial_capacity() {
        let allocator = KGlobalAllocator::new();
        let mut kvec = KVec::<usize, KGlobalAllocator>::with_capacity(&allocator, 0);

        kvec.push(1);
        kvec.push(1);
        kvec.push(1);
        kvec.push(1);
        kvec.push(1);

        assert_eq!(1, kvec.pop().unwrap());
        assert_eq!(1, kvec.pop().unwrap());
        assert_eq!(1, kvec.pop().unwrap());
        assert_eq!(1, kvec.pop().unwrap());
        assert_eq!(1, kvec.pop().unwrap());
    }

    #[test]
    fn test_kvec_extend_from_slice() {
        let allocator = KGlobalAllocator::new();
        let mut kvec = KVec::<usize, KGlobalAllocator>::new(&allocator);
        assert_eq!(0, kvec._capacity);
        assert_eq!(0, kvec._length);
        
        kvec.extend_from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(5, kvec._capacity);
        assert_eq!(5, kvec._length);
        
        kvec.extend_from_slice(&[6, 7, 8, 9, 10]);
        assert_eq!(10, kvec._capacity);
        assert_eq!(10, kvec._length);

        kvec.extend_from_slice(&[11, 12, 13, 14, 15]);
        assert_eq!(15, kvec._capacity);
        assert_eq!(15, kvec._length);

        let total_kvec = KVec::from_slice(&allocator, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(total_kvec, kvec);
        assert_eq!(15, kvec._capacity);
        assert_eq!(15, kvec._length);
    }

    #[test]
    fn test_kvec_clone() {
        let allocator = KGlobalAllocator::new();
        let mut kvec1 = KVec::<usize, KGlobalAllocator>::new(&allocator);

        kvec1.push(1);
        kvec1.push(2);
        kvec1.push(3);
        kvec1.push(4);
        kvec1.push(5);

        let kvec2 = kvec1.clone();
        assert_eq!(kvec1, kvec2);

        assert_eq!(kvec1._capacity, kvec2._capacity);
        assert_eq!(kvec1._length, kvec2._length);
        assert_ne!(kvec1._buffer.addr(), kvec2._buffer.addr());
    }

    #[test]
    fn test_kvec_drop_is_called() {
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

            let mut kvec = KVec::new(&allocator);
            kvec.push(MustBeDropped(dropflag));
        }

        assert_eq!(true, dropflag.load(Ordering::Relaxed));
    }

    #[test]
    fn test_kvec_drop_is_called_on_popped_element() {
        let allocator = KGlobalAllocator::new();
        let dropflag = Arc::new(AtomicBool::new(false));
        struct MustBeDropped(Arc<AtomicBool>);

        impl Drop for MustBeDropped {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }

        let mut kvec = KVec::new(&allocator);

        {
            let dropflag = Arc::clone(&dropflag);
            kvec.push(MustBeDropped(dropflag));
        }

        {
            let _ = kvec.pop().unwrap();
        }

        assert_eq!(true, dropflag.load(Ordering::Relaxed));
        assert!(kvec.is_empty());
    }


    #[test]
    fn test_kvec_remove_start() {
        let allocator = KGlobalAllocator::new();
        let mut kvec = KVec::<usize, KGlobalAllocator>::new(&allocator);

        kvec.push(1);
        kvec.push(2);
        kvec.push(3);
        kvec.push(4);
        kvec.push(5);

        assert_eq!(1, kvec.remove(0).unwrap());
        assert_eq!(2, kvec.remove(0).unwrap());
        assert_eq!(3, kvec.remove(0).unwrap());
        assert_eq!(4, kvec.remove(0).unwrap());
        assert_eq!(5, kvec.remove(0).unwrap());
        assert!(kvec.remove(0).is_none());
        assert!(kvec.remove(999).is_none());
    }

    #[test]
    fn test_kvec_remove_end() {
        let allocator = KGlobalAllocator::new();
        let mut kvec = KVec::<usize, KGlobalAllocator>::new(&allocator);

        kvec.push(1);
        kvec.push(2);
        kvec.push(3);
        kvec.push(4);
        kvec.push(5);

        assert_eq!(5, kvec.remove(4).unwrap());
        assert_eq!(4, kvec.remove(3).unwrap());
        assert_eq!(3, kvec.remove(2).unwrap());
        assert_eq!(2, kvec.remove(1).unwrap());
        assert_eq!(1, kvec.remove(0).unwrap());
        assert!(kvec.remove(0).is_none());
        assert!(kvec.remove(999).is_none());
    }

    #[test]
    fn test_kvec_swap() {
        let allocator = KGlobalAllocator::new();
        let mut kvec = KVec::<usize, KGlobalAllocator>::new(&allocator);

        kvec.push(1);
        kvec.push(2);

        kvec.swap(0, 1);

        assert_eq!(2, kvec[0]);
        assert_eq!(1, kvec[1]);
    }
}
