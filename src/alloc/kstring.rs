use std::{ops::{Index, IndexMut}, str::Utf8Error};

use crate::{alloc::kallocator::Allocator, collection::kvec::KVec};

pub struct KString<'a, A : Allocator> {
    _allocator: &'a A,
    _data: KVec<'a, u8, A>
}

#[derive(Debug)]
pub struct KFromUtf8Error<'a, A : Allocator> {
    bytes: KVec<'a, u8, A>,
    error: Utf8Error,
}

impl<'a, A : Allocator> KString<'a, A> {
    pub fn new(allocator: &'a A) -> Self {
        return Self {
            _allocator: allocator,
            _data: KVec::new(allocator)
        }
    }

    pub fn with_capacity(allocator: &'a A, capacity: usize) -> Self {
        return Self {
            _allocator: allocator,
            _data: KVec::with_capacity(allocator, capacity)
        }
    }

    pub fn from_str(allocator: &'a A, s: &str) -> Result<Self, KFromUtf8Error<'a, A>> {
        let kvec = {
            let mut kvec: KVec<u8, A> = KVec::with_capacity(allocator, s.len());

            for byte in s.as_bytes() {
                kvec.push(*byte);
            }

            kvec
        };
        
        return match str::from_utf8(&kvec) {
            Ok(..) => Ok(Self { _allocator: allocator, _data: kvec }),
            Err(e) => Err(KFromUtf8Error { bytes: kvec, error: e }),
        };
    }

    pub fn len(&self) -> usize {
        return self._data.len();
    }

    pub fn push(&mut self, byte: u8) {
        self._data.push(byte);
    }

    pub fn push_str(&mut self, s: &str) {
        self._data.extend_from_slice(s.as_bytes())
    }

    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(&self._data.as_ref())
    }
}

impl<'a, A : Allocator> Index<usize> for KString<'a, A> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self._data[index]
    }
}

impl<'a, A : Allocator> IndexMut<usize> for KString<'a, A> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self._data[index]
    }
}


mod test {
    use crate::alloc::{global::GlobalAllocator, kstring::KString};

    #[test]
    fn kstring_push() {
        let allocator = GlobalAllocator::new();
        let mut kstring = KString::new(&allocator);
        
        kstring.push(b'H');
        kstring.push(b'e');
        kstring.push(b'l');
        kstring.push(b'l');
        kstring.push(b'o');
        kstring.push(b',');
        kstring.push(b' ');
        kstring.push_str("World!");

        assert_eq!("Hello, World!", kstring.as_str().unwrap());
    }

    #[test]
    fn kstring_from_str() {
        let allocator = GlobalAllocator::new();
        let kstring = KString::from_str(&allocator, "Hello, World!").unwrap();

        assert_eq!(13, kstring.len());
        assert_eq!(b'H', kstring[0]);
        assert_eq!(b'e', kstring[1]);
        assert_eq!(b'l', kstring[2]);
        assert_eq!(b'l', kstring[3]);
        assert_eq!(b'o', kstring[4]);
        assert_eq!(b',', kstring[5]);
        assert_eq!(b' ', kstring[6]);
        assert_eq!(b'W', kstring[7]);
        assert_eq!(b'o', kstring[8]);
        assert_eq!(b'r', kstring[9]);
        assert_eq!(b'l', kstring[10]);
        assert_eq!(b'd', kstring[11]);
        assert_eq!(b'!', kstring[12]);
    }
}
