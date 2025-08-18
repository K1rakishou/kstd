use std::{alloc::{alloc, dealloc, realloc, Layout}, ops::{Deref, DerefMut, Index, IndexMut}, ptr::NonNull};
use std::fmt::Debug;

pub struct KVec<T> {
    _buffer: NonNull<T>,
    _capacity: usize,
    _length: usize
}

impl<T> KVec<T> {
    pub fn new() -> Self {
        return Self {
            _buffer: NonNull::dangling(),
            _capacity: 0,
            _length: 0
        }
    }

    pub fn push(&mut self, value: T) {
        if self._capacity <= self._length {
            self.grow();
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
            self._buffer
                .offset(index as isize)
                .read()
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

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_ref().iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.as_mut().iter_mut()
    }

    fn grow(&mut self) {
        let elem_size = std::mem::size_of::<T>();
        let elem_align = std::mem::align_of::<T>();

        if elem_size == 0 {
            panic!("ZSTs are not supported yet!");
        }
        
        let new_capacity = match self._capacity {
            0 => 4,
            cap => cap.checked_mul(2).expect("Capacity overflow"),
        };

        let new_size = new_capacity
            .checked_mul(elem_size)
            .expect("Size overflow");

        let new_buffer = unsafe {
            let ptr_raw = match self._capacity {
                0 => {
                    let new_layout = Layout::from_size_align(new_size, elem_align).unwrap();

                    alloc(new_layout) as *mut T
                }
                _ => {
                    let old_size = self._capacity
                        .checked_mul(elem_size)
                        .expect("Size overflow");
                    let old_layout = Layout::from_size_align(old_size, elem_align).unwrap();
                    
                    let ptr = self._buffer.as_ptr() as *mut u8;
                    realloc(ptr, old_layout, new_size) as *mut T
                }
            };

            assert_ne!(true, ptr_raw.is_null());            
            NonNull::new_unchecked(ptr_raw)
        };

        self._buffer = new_buffer;
        self._capacity = new_capacity;
    }
}

impl<T> Drop for KVec<T> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(self._buffer.as_ptr());

            let layout = Layout::new::<T>();
            dealloc(self._buffer.as_ptr() as *mut u8, layout);
        }
    }
}

impl<T> Index<usize> for KVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<usize> for KVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T> Debug for KVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KVec(")?;
        write!(f, "buffer: 0x{:p}, ", self._buffer.as_ptr())?;
        write!(f, "capacity: {}, ", self._capacity)?;
        write!(f, "length: {}", self._length)?;
        write!(f, ")")?;
        return Ok(());
    }
}

impl<T> Deref for KVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        return unsafe { std::slice::from_raw_parts(self._buffer.as_ptr(), self._length) };
    }
}

impl<T> DerefMut for KVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return unsafe { std::slice::from_raw_parts_mut(self._buffer.as_ptr(), self._length) };
    }
}

impl<T> AsRef<[T]> for KVec<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for KVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

mod test {
    use super::KVec;

    #[test]
    fn test_kvec() {
        let mut kvec = KVec::<u64>::new();

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
        fn accepts_slice(slice: &[usize]) {
            println!("{:?}", slice);
        }

        fn accepts_slice_mut(slice_mut: &mut[usize]) {
            println!("{:?}", slice_mut);
        }

        let mut kvec = KVec::<usize>::new();
        kvec.push(11223344);

        accepts_slice(&kvec);
        accepts_slice_mut(&mut kvec);
    }

    #[test]
    fn test_kvec_iter() {
        let mut kvec = KVec::<usize>::new();
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
}
