use std::{ops::{Index, IndexMut}, str::Utf8Error};

use crate::{alloc::kallocator::KAllocator, collection::kvec::KVec};

pub struct KString<'allocator, A : KAllocator> {
    _allocator: &'allocator A,
    _vec: KVec<'allocator, u8, A>
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct KFromUtf8Error<'allocator, A : KAllocator> {
    bytes: KVec<'allocator, u8, A>,
    error: Utf8Error,
}

#[allow(dead_code)]
impl<'allocator, A : KAllocator> KString<'allocator, A> {
    pub fn new(allocator: &'allocator A) -> Self {
        return Self {
            _allocator: allocator,
            _vec: KVec::new(allocator)
        }
    }

    pub fn with_capacity(allocator: &'allocator A, capacity: usize) -> Self {
        return Self {
            _allocator: allocator,
            _vec: KVec::with_capacity(allocator, capacity)
        }
    }

    pub fn from_str(allocator: &'allocator A, s: &str) -> Result<Self, KFromUtf8Error<'allocator, A>> {
        let kvec = {
            let mut kvec: KVec<u8, A> = KVec::with_capacity(allocator, s.len());

            for byte in s.as_bytes() {
                kvec.push(*byte);
            }

            kvec
        };
        
        return match str::from_utf8(&kvec) {
            Ok(..) => Ok(Self { _allocator: allocator, _vec: kvec }),
            Err(e) => Err(KFromUtf8Error { bytes: kvec, error: e }),
        };
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self._vec.len();
    }

    pub fn push(&mut self, byte: u8) {
        self._vec.push(byte);
    }

    pub fn push_char(&mut self, ch: char) {
        let mut buffer = [0u8; 4];
        let length = ch.encode_utf8(&mut buffer).len();

        self._vec.extend_from_slice(&buffer[0..length]);
    }

    pub fn push_str(&mut self, s: &str) {
        self._vec.extend_from_slice(s.as_bytes())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        return unsafe { str::from_utf8_unchecked(&self._vec.as_ref()) };
    }
}

impl<'allocator, A : KAllocator> Index<usize> for KString<'allocator, A> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self._vec[index]
    }
}

impl<'allocator, A : KAllocator> IndexMut<usize> for KString<'allocator, A> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self._vec[index]
    }
}

impl<'allocator, A : KAllocator> core::fmt::Write for KString<'allocator, A> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_str(s);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.push_char(c);
        Ok(())
    }
}

// TODO: impl Deref, DerefMut, AsRef<str>

#[allow(unused_imports)]
mod test {
    use crate::alloc::{kglobal_allocator::KGlobalAllocator, kstring::KString};

    #[test]
    fn kstring_push() {
        let allocator = KGlobalAllocator::new();
        let mut kstring = KString::new(&allocator);

        kstring.push(b'H');
        kstring.push(b'e');
        kstring.push(b'l');
        kstring.push(b'l');
        kstring.push(b'o');
        kstring.push(b',');
        kstring.push(b' ');
        kstring.push_str("World!");

        assert_eq!("Hello, World!", kstring.as_str());
    }

    #[test]
    fn kstring_from_str() {
        let allocator = KGlobalAllocator::new();
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
