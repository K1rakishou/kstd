use core::{alloc::Layout, ops::{Deref, DerefMut, Index, IndexMut}, ptr::NonNull, slice};
use core::fmt::Debug;
use std::cmp::Ordering;

use crate::alloc::kallocator::KAllocator;

pub struct KVec<'a, T, A : KAllocator> {
    _allocator: &'a A,
    _buffer: NonNull<T>,
    _capacity: usize,
    _length: usize
}

impl<'a, T, A : KAllocator> KVec<'a, T, A> {
    pub fn new(allocator: &'a A) -> Self {
        return Self {
            _allocator: allocator,
            _buffer: NonNull::dangling(),
            _capacity: 0,
            _length: 0
        }
    }

    pub fn with_capacity(allocator: &'a A, capacity: usize) -> Self {
        let (new_buffer, new_capacity) = Self::grow(allocator, NonNull::dangling(), capacity, true);
        let this = Self {
            _allocator: allocator,
            _buffer: new_buffer,
            _capacity: new_capacity,
            _length: 0
        };

        return this;
    }

    pub fn from_slice(allocator: &'a A, slice: &[T]) -> Self {
        let (new_buffer, new_capacity) = Self::grow(allocator, NonNull::dangling(), slice.len(), true);
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
            let (new_buffer, new_capacity) = Self::grow(self._allocator, self._buffer, self._capacity, self.is_empty());

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

    pub fn extend_from_slice(&mut self, slice: &[T]) {
        let slice_len = slice.len();
        let available_len = self._length as isize - self._capacity as isize;

        if available_len < slice_len as isize {
            let (new_buffer, new_capacity) = Self::grow(self._allocator, self._buffer, self._capacity + slice_len, self.is_empty());
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

    fn grow(allocator: &'a A, buffer: NonNull<T>, capacity: usize, is_buffer_empty: bool) -> (NonNull<T>, usize) {
        let elem_size = core::mem::size_of::<T>();
        let elem_align = core::mem::align_of::<T>();

        if elem_size == 0 {
            panic!("ZSTs are not supported yet!");
        }
        
        let new_capacity = if is_buffer_empty && capacity > 0 {
            capacity
        } else {
            match capacity {
                0 => 4,
                cap => {
                    if cap == usize::MAX {
                        panic!("Capacity overflow");
                    }

                    cap.saturating_mul(2)
                },
            }
        };

        let new_size = new_capacity
            .checked_mul(elem_size)
            .expect("Size overflow");

        let new_buffer = unsafe {
            let ptr_raw = if is_buffer_empty {
                // length == 0 is the very first allocation. At this point buffer pointer is dangling so we need to allocate it.
                let new_layout = Layout::from_size_align(new_size, elem_align).unwrap();
                let new_ptr = allocator.allocate(new_layout);

                new_ptr
            } else {
                // length != 0 means that we are reallocating which means it's safe to access buffer's pointer.
                let new_layout = Layout::from_size_align(new_size, elem_align).unwrap();
                let old_ptr = buffer.as_ptr() as *mut u8;
                let new_ptr = allocator.reallocate(old_ptr, new_layout);

                new_ptr
            };

            let Some(ptr_raw) = ptr_raw else {
                todo!("Try to defragment the memory in the allocator, or something");
            };

            NonNull::new_unchecked(ptr_raw as *mut T)
        };

        return (new_buffer, new_capacity);
    }
}

impl<'a, T, A : KAllocator> Drop for KVec<'a, T, A> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self._buffer.as_ptr());
            self._allocator.deallocate(self._buffer.as_ptr() as *mut u8);
        }
    }
}

impl<'a, T, A : KAllocator> Index<usize> for KVec<'a, T, A> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<'a, T, A : KAllocator> IndexMut<usize> for KVec<'a, T, A> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<'a, T, A : KAllocator> Debug for KVec<'a, T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "KVec(")?;
        write!(f, "buffer: 0x{:p}, ", self._buffer.as_ptr())?;
        write!(f, "capacity: {}, ", self._capacity)?;
        write!(f, "length: {}", self._length)?;
        write!(f, ")")?;
        return Ok(());
    }
}

impl<'a, T, A : KAllocator> Deref for KVec<'a, T, A> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        return unsafe { core::slice::from_raw_parts(self._buffer.as_ptr(), self._length) };
    }
}

impl<'a, T, A : KAllocator> DerefMut for KVec<'a, T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return unsafe { core::slice::from_raw_parts_mut(self._buffer.as_ptr(), self._length) };
    }
}

impl<'a, T, A : KAllocator> AsRef<[T]> for KVec<'a, T, A> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<'a, T, A : KAllocator> AsMut<[T]> for KVec<'a, T, A> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'a, T, A : KAllocator> IntoIterator for KVec<'a, T, A> {
    type Item = T;
    type IntoIter = KVecIterator<'a, T, A>;

    fn into_iter(self) -> Self::IntoIter {
        return KVecIterator::new(self);
    }
}

pub struct KVecIterator<'a, T, A : KAllocator> {
    _kvec: KVec<'a, T, A>,
    _index: usize
}

impl<'a, T, A : KAllocator> KVecIterator<'a, T, A> {
    pub fn new(kvec: KVec<'a, T, A>) -> Self {
        return Self {
            _kvec: kvec,
            _index: 0
        };
    }
}

impl<'a, T, A : KAllocator> Iterator for KVecIterator<'a, T, A> {
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

impl<'a, T : PartialEq, A : KAllocator> PartialEq for KVec<'a, T, A> {
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

impl<'a, T : PartialOrd, A : KAllocator> PartialOrd for KVec<'a, T, A> {
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


mod test {
    use crate::alloc::global::GlobalAllocator;
    use super::KVec;

    #[test]
    fn test_kvec() {
        let allocator = GlobalAllocator::new();
        let mut kvec = KVec::<u64, GlobalAllocator>::new(&allocator);

        for i in 0..1024 {
            kvec.push(i);
        }

        for i in 0..1024 {
            assert_eq!(i, kvec[i] as usize);
            assert_eq!(i, *kvec.get_mut(i).unwrap() as usize);
        }

        for i in (0..1024).rev() {
            assert_eq!(i, kvec.pop().unwrap());
        }

        assert_eq!(None, kvec.pop());
    }

    #[test]
    fn test_kvec_deref() {
        let allocator = GlobalAllocator::new();

        fn accepts_slice(slice: &[usize]) {
            let _ = slice;
        }

        fn accepts_slice_mut(slice_mut: &mut[usize]) {
            let _ = slice_mut;
        }

        let mut kvec = KVec::<usize, GlobalAllocator>::new(&allocator);
        kvec.push(11223344);

        accepts_slice(&kvec);
        accepts_slice_mut(&mut kvec);
    }

    #[test]
    fn test_kvec_iter() {
        let allocator = GlobalAllocator::new();
        let mut kvec = KVec::<usize, GlobalAllocator>::new(&allocator);
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
        let allocator = GlobalAllocator::new();
        let mut kvec = KVec::<usize, GlobalAllocator>::new(&allocator);
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
        let allocator = GlobalAllocator::new();

        let mut kvec = KVec::<usize, GlobalAllocator>::with_capacity(&allocator, 4);
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
        let allocator = GlobalAllocator::new();
        let mut kvec = KVec::<usize, GlobalAllocator>::with_capacity(&allocator, 0);

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
        let allocator = GlobalAllocator::new();
        let mut kvec = KVec::<usize, GlobalAllocator>::with_capacity(&allocator, 0);

        kvec.extend_from_slice(&[1, 2, 3, 4, 5]);
        kvec.extend_from_slice(&[6, 7, 8, 9, 10]);
        kvec.extend_from_slice(&[11, 12, 13, 14, 15]);

        let total_kvec = KVec::from_slice(&allocator, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(total_kvec, kvec);
    }
}
