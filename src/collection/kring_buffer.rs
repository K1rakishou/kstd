use core::fmt::{Debug, Formatter};
use core::ptr::NonNull;
use core::alloc::Layout;
use crate::alloc::kallocator::KAllocator;
use crate::collection::layout_from_capacity;

pub struct KRingBuffer<'allocator, T, A : KAllocator> {
    _allocator: &'allocator A,
    _buffer: NonNull<T>,
    _capacity: usize,
    _length: usize,
    _index: usize
}

impl<'allocator, T, A : KAllocator> KRingBuffer<'allocator, T, A> {
    pub fn new(allocator: &'allocator A, capacity: usize) -> Self {
        let elem_size = core::mem::size_of::<T>();
        if elem_size == 0 {
            panic!("ZSTs are not supported yet!");
        }

        let layout = Layout::array::<T>(capacity).unwrap();

        let Some(memory) = allocator.allocate(layout) else {
            panic!("Failed to allocate memory for layout: {:?}", layout);
        };
        
        let buffer = unsafe { NonNull::new_unchecked(memory as *mut T) };
        
        return Self {
            _allocator: allocator,
            _buffer: buffer,
            _capacity: capacity,
            _length: 0,
            _index: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        let index = self._index % self._capacity;
        self.write(index, value);

        self._index += 1;
        self._length = core::cmp::min(self._length + 1, self._capacity);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self._index == 0 {
            return None;
        }

        let index = (self._index - 1) % self._length;
        let value = unsafe {
            let element_ptr = self._buffer
                .offset(index as isize)
                .as_ptr();

            core::ptr::read(element_ptr)
        };

        self._index -= 1;
        self._length = core::cmp::max(self._length - 1, 0);

        return Some(value);
    }

    #[inline]
    pub fn length(&self) -> usize {
        return self._length;
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if !self.is_within_range(index) {
            return None;
        }

        let index = (self._index + index) % self._length;
        return Some(self.read(index));
    }

    #[inline]
    pub fn first(&self) -> Option<&T> {
        if self._length == 0 {
            return None;
        }

        let index = (self._index as isize) - (self._length as isize);
        if index < 0 {
            return None;
        }

        let index = index as usize % self._length;
        return Some(self.read(index));
    }

    #[inline]
    pub fn last(&self) -> Option<&T> {
        if self._length == 0 {
            return None;
        }

        let index = self._index as isize - 1;
        if index < 0 {
            return None;
        }

        let index = index as usize % self._length;
        return Some(self.read(index));
    }

    #[inline]
    pub fn iter(&self) -> KRingBufferIterator<'_, T, A> {
        return KRingBufferIterator::new(self);
    }

    #[inline]
    fn is_within_range(&self, index: usize) -> bool {
        if self._length == 0 {
            return false;
        }

        if index < self._length {
            return true;
        }

        let start_range = (index % self._length)..(self._length - 1);
        if start_range.contains(&index) {
            return true;
        }

        let end_range = 0..(index % self._length);
        if end_range.contains(&index) {
            return true;
        }

        return false;
    }

    #[inline]
    fn read(&self, index: usize) -> &T {
        // Can read up to the length (inclusive)
        if index >= self._length {
            panic!("Index out of bounds");
        }

        return unsafe { &*self._buffer.as_ptr().offset(index as isize) };
    }

    #[inline]
    fn write(&self, index: usize, value: T) {
        // Can write up to the capacity (inclusive)
        if index >= self._capacity {
            panic!("Index out of bounds");
        }

        unsafe { self._buffer.as_ptr().offset(index as isize).write(value) };
    }
}

impl<'allocator, T, A : KAllocator> Drop for KRingBuffer<'allocator, T, A> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>() {
            for offset in 0 .. self._capacity {
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

impl<'allocator, T : Clone, A : KAllocator> KRingBuffer<'allocator, T, A> {
    pub fn clone(&self) -> KRingBuffer<'allocator, T, A> {
        let layout = Layout::array::<T>(self._capacity).unwrap();
        let Some(memory) = self._allocator.allocate(layout) else {
            panic!("Failed to allocate memory for layout: {:?}", layout);
        };

        let buffer = unsafe {
            let buffer = NonNull::new_unchecked(memory as *mut T);
            buffer.copy_from_nonoverlapping(self._buffer, layout.size());
            buffer
        };

        return KRingBuffer {
            _allocator: self._allocator,
            _buffer: buffer,
            _capacity: self._capacity,
            _length: self._length,
            _index: self._index
        }
    }
}

impl<'allocator, T : Clone, A : KAllocator> Clone for KRingBuffer<'allocator, T, A> {
    fn clone(&self) -> Self {
        return KRingBuffer::clone(self)
    }
}

impl<'allocator, T : Debug, A : KAllocator> Debug for KRingBuffer<'allocator, T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "RingBuffer{{")?;
        write!(f, "length: {length}, ", length = self._length)?;
        write!(f, "index: {index}", index = self._index)?;
        write!(f, "}}")?;

        return Ok(());
    }
}

pub struct KRingBufferIterator<'a, T, A : KAllocator> {
    _krb: &'a KRingBuffer<'a, T, A>,
    _iterator_index: usize,
}

impl<'a, T, A : KAllocator> KRingBufferIterator<'a, T, A> {
    pub fn new(krb: &'a KRingBuffer<'a, T, A>) -> Self {
        return Self {
            _krb: krb,
            _iterator_index: 0,
        }
    }
}

impl<'a, T, A : KAllocator> Iterator for KRingBufferIterator<'a, T, A> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let iterator_index = self._iterator_index;
        self._iterator_index += 1;

        return self._krb.get(iterator_index);
    }
}

mod test {
    use crate::alloc::kglobal_allocator::KGlobalAllocator;
    use crate::collection::kring_buffer::KRingBuffer;

    #[test]
    fn test_ring_buffer_with_overflow() {
        let allocator = KGlobalAllocator::new();
        let mut krb = KRingBuffer::new(&allocator, 5);

        assert_eq!(0, krb.length());
        krb.push(1);
        krb.push(2);
        krb.push(3);
        krb.push(4);

        let values = krb.iter().cloned().collect::<Vec<_>>();
        assert_eq!(values, &[1, 2, 3, 4]);
        assert_eq!(1, *krb.first().unwrap());
        assert_eq!(4, *krb.last().unwrap());
        assert_eq!(1, *krb.get(0).unwrap());
        assert_eq!(2, *krb.get(1).unwrap());
        assert_eq!(3, *krb.get(2).unwrap());
        assert_eq!(4, *krb.get(3).unwrap());
        assert!(krb.get(4).is_none());
        assert_eq!(4, krb.length());

        krb.push(5);
        krb.push(6);

        let values = krb.iter().cloned().collect::<Vec<_>>();
        assert_eq!(values, &[2, 3, 4, 5, 6]);
        assert_eq!(2, *krb.first().unwrap());
        assert_eq!(6, *krb.last().unwrap());
        assert_eq!(2, *krb.get(0).unwrap());
        assert_eq!(3, *krb.get(1).unwrap());
        assert_eq!(4, *krb.get(2).unwrap());
        assert_eq!(5, *krb.get(3).unwrap());
        assert_eq!(6, *krb.get(4).unwrap());
        assert_eq!(5, krb.length());

        krb.push(7);
        krb.push(8);
        krb.push(9);
        krb.push(10);
        krb.push(11);

        let values = krb.iter().cloned().collect::<Vec<_>>();
        assert_eq!(values, &[7, 8, 9, 10, 11]);
        assert_eq!(7, *krb.first().unwrap());
        assert_eq!(11, *krb.last().unwrap());
        assert_eq!(7, *krb.get(0).unwrap());
        assert_eq!(8, *krb.get(1).unwrap());
        assert_eq!(9, *krb.get(2).unwrap());
        assert_eq!(10, *krb.get(3).unwrap());
        assert_eq!(11, *krb.get(4).unwrap());
        assert_eq!(5, krb.length());

        assert!(krb.get(123).is_none());
    }

    #[test]
    fn test_ring_buffer_pop() {
        let allocator = KGlobalAllocator::new();
        let mut krb = KRingBuffer::new(&allocator, 5);

        assert!(krb.pop().is_none());
        assert!(krb.first().is_none());
        assert!(krb.last().is_none());
        assert_eq!(0, krb.length());

        krb.push(1);
        krb.push(2);
        krb.push(3);
        krb.push(4);

        assert_eq!(4, krb.pop().unwrap());
        assert_eq!(3, krb.pop().unwrap());
        assert_eq!(2, krb.pop().unwrap());
        assert_eq!(1, krb.pop().unwrap());
        assert!(krb.pop().is_none());
        assert!(krb.first().is_none());
        assert!(krb.last().is_none());
        assert_eq!(0, krb.length());
    }
}