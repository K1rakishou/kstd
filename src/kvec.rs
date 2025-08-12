use std::{alloc::{alloc, dealloc, realloc, Layout}, ptr::NonNull};

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

mod test {
    use super::KVec;

    #[test]
    fn test_kvec() {
        let mut kvec = KVec::<u64>::new();

        for i in 0..1024 {
            kvec.push(i);
        }

        for i in (0..1024).rev() {
            assert_eq!(i, kvec.pop().unwrap());
        }

        assert_eq!(None, kvec.pop());
    }
    
}
