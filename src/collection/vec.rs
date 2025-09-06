use core::{alloc::{self, Layout}, ops::{Deref, DerefMut, Index, IndexMut}, ptr::NonNull};
use core::fmt::Debug;

use crate::alloc::allocator::Allocator;

pub struct KVec<'a, T, A : Allocator> {
    _allocator: &'a A,
    _buffer: NonNull<T>,
    _capacity: usize,
    _length: usize
}

impl<'a, T, A : Allocator> KVec<'a, T, A> {
    pub fn new(allocator: &'a A) -> Self {
        return Self {
            _allocator: allocator,
            _buffer: NonNull::dangling(),
            _capacity: 0,
            _length: 0
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        // assert!(capacity > 0);

        // let (new_buffer, new_capacity) = Self::grow(NonNull::dangling(), capacity);
        // let this = Self {
        //     _buffer: new_buffer,
        //     _capacity: new_capacity,
        //     _length: 0
        // };

        // return this;
        todo!("Doesn't work because growing memory will access dangling memory pointer, need to fix that first")
    }

    pub fn push(&mut self, value: T) {
        if self._capacity <= self._length {
            let (new_buffer, new_capacity) = Self::grow(self._allocator, self._buffer, self._capacity);

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
        if self._length == 0 {
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

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_ref().iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.as_mut().iter_mut()
    }

    pub fn len(&self) -> usize {
        return self._length;
    }

    pub fn last_index(&self) -> Option<usize> {
        return self._length.checked_sub(1);
    }

    pub fn last(&self) -> Option<&T> {
        if self._length == 0 {
            return None;
        }
        
        let last_element = unsafe { self._buffer.add(self._length).as_ref() };
        return Some(last_element);
    }

    fn grow(allocator: &'a A, buffer: NonNull<T>, capacity: usize) -> (NonNull<T>, usize) {
        let elem_size = core::mem::size_of::<T>();
        let elem_align = core::mem::align_of::<T>();

        if elem_size == 0 {
            panic!("ZSTs are not supported yet!");
        }
        
        let new_capacity = match capacity {
            0 => 4,
            cap => cap.checked_mul(2).expect("Capacity overflow"),
        };

        let new_size = new_capacity
            .checked_mul(elem_size)
            .expect("Size overflow");

        let new_buffer = unsafe {
            let ptr_raw = match capacity {
                0 => {
                    let new_layout = Layout::from_size_align(new_size, elem_align).unwrap();
                    let new_ptr = allocator.allocate(new_layout);

                    new_ptr
                }
                _ => {
                    let new_layout = Layout::from_size_align(new_size, elem_align).unwrap();
                    let old_ptr = buffer.as_ptr() as *mut u8;
                    let new_ptr = allocator.reallocate(old_ptr, new_layout);

                    new_ptr
                }
            };

            let Some(ptr_raw) = ptr_raw else {
                todo!("Try to defragment the memory in the allocator, or something");
            };

            NonNull::new_unchecked(ptr_raw as *mut T)
        };

        return (new_buffer, new_capacity);
    }
}

impl<'a, T, A : Allocator> Drop for KVec<'a, T, A> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self._buffer.as_ptr());
            self._allocator.deallocate(self._buffer.as_ptr() as *mut u8);
        }
    }
}

impl<'a, T, A : Allocator> Index<usize> for KVec<'a, T, A> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<'a, T, A : Allocator> IndexMut<usize> for KVec<'a, T, A> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<'a, T, A : Allocator> Debug for KVec<'a, T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "KVec(")?;
        write!(f, "buffer: 0x{:p}, ", self._buffer.as_ptr())?;
        write!(f, "capacity: {}, ", self._capacity)?;
        write!(f, "length: {}", self._length)?;
        write!(f, ")")?;
        return Ok(());
    }
}

impl<'a, T, A : Allocator> Deref for KVec<'a, T, A> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        return unsafe { core::slice::from_raw_parts(self._buffer.as_ptr(), self._length) };
    }
}

impl<'a, T, A : Allocator> DerefMut for KVec<'a, T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return unsafe { core::slice::from_raw_parts_mut(self._buffer.as_ptr(), self._length) };
    }
}

impl<'a, T, A : Allocator> AsRef<[T]> for KVec<'a, T, A> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<'a, T, A : Allocator> AsMut<[T]> for KVec<'a, T, A> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'a, T, A : Allocator> IntoIterator for KVec<'a, T, A> {
    type Item = T;
    type IntoIter = KVecIterator<'a, T, A>;

    fn into_iter(self) -> Self::IntoIter {
        return KVecIterator::new(self);
    }
}

pub struct KVecIterator<'a, T, A : Allocator> {
    _kvec: KVec<'a, T, A>,
    _index: usize
}

impl<'a, T, A : Allocator> KVecIterator<'a, T, A> {
    pub fn new(kvec: KVec<'a, T, A>) -> Self {
        return Self {
            _kvec: kvec,
            _index: 0
        };
    }
}

impl<'a, T, A : Allocator> Iterator for KVecIterator<'a, T, A> {
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
            println!("{:?}", slice);
        }

        fn accepts_slice_mut(slice_mut: &mut[usize]) {
            println!("{:?}", slice_mut);
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
}
