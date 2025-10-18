use std::hash::{BuildHasher, Hash, RandomState};
use crate::{alloc::kallocator::KAllocator, collection::kvec::KVec, kformatln};

enum ResizeMode {
    Up,
    Down
}

pub struct KHashMap<'allocator, K : Hash + PartialEq, V : PartialEq, A : KAllocator, BH : BuildHasher> {
    _allocator: &'allocator A,
    _bucket_array: KBucketArray<'allocator, K, V, A, BH>,
    _min_load_factor: f32,
    _max_load_factor: f32,
}

pub struct KHashMapOptions<BH : BuildHasher> {
    min_load_factor: f32,
    max_load_factor: f32,
    // capacity == 0 means the default KVec capacity will be used
    capacity: usize,
    build_hasher: BH,
}

#[allow(dead_code)]
impl<'allocator, K : Hash + PartialEq, V : PartialEq, A : KAllocator> KHashMap<'allocator, K, V, A, RandomState> {
    pub fn new(allocator: &'allocator A) -> Self {
        let options: KHashMapOptions<RandomState> = KHashMapOptions::default();

        return Self {
            _allocator: allocator,
            _bucket_array: KBucketArray::new(allocator, options.capacity, options.build_hasher),
            _min_load_factor: options.min_load_factor,
            _max_load_factor: options.max_load_factor,
        };
    }
}

#[allow(dead_code)]
impl<'allocator, K : Hash + PartialEq, V : PartialEq, A : KAllocator, BH : BuildHasher> KHashMap<'allocator, K, V, A, BH> {
    pub fn with_options(allocator: &'allocator A, options: KHashMapOptions<BH>) -> Self {
        options.ensure_options_valid();

        return Self {
            _allocator: allocator,
            _bucket_array: KBucketArray::new(allocator, options.capacity, options.build_hasher),
            _min_load_factor: options.min_load_factor,
            _max_load_factor: options.max_load_factor,
        };
    }
}

#[allow(dead_code)]
impl<'allocator, K : Hash + PartialEq, V : PartialEq, A : KAllocator, BH : BuildHasher> KHashMap<'allocator, K, V, A, BH> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        return self._bucket_array.insert(key, value, self._max_load_factor);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        return self._bucket_array.get(key);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        return self._bucket_array.remove(key, self._min_load_factor);
    }

    pub fn length(&self) -> usize {
        return self._bucket_array.iter().fold(0, |acc, bucket| acc + bucket.occupied_entries_length());
    }
}

struct KBucketArray<'allocator, K : Hash + PartialEq, V : PartialEq, A : KAllocator, BH : BuildHasher> {
    _allocator: &'allocator A,
    _buckets: KVec<'allocator, KBucket<'allocator, K, V, A>, A>,
    _build_hasher: BH
}

#[allow(dead_code)]
impl<'allocator, K : Hash + PartialEq, V : PartialEq, A : KAllocator, BH : BuildHasher> KBucketArray<'allocator, K, V, A, BH> {
    fn new(allocator: &'allocator A, capacity: usize, build_hasher: BH) -> Self {
        let buckets = if capacity > 0 {
            KVec::with_capacity(allocator, capacity)
        } else {
            KVec::new(allocator)
        };

        return Self {
            _allocator: allocator,
            _buckets: buckets,
            _build_hasher: build_hasher
        };
    }

    fn insert(&mut self, key: K, value: V, max_load_factor: f32) -> Option<V> {
        if self._buckets.is_empty() || self.load_factor() > max_load_factor {
            self.resize(ResizeMode::Up);
        }

        let bucket_index = Self::bucket_index(&self._build_hasher, &key, self._buckets.len());
        let Some(bucket) = self.get_bucket_mut(bucket_index) else {
            return None;
        };

        return bucket.insert(key, value).map(|(_, v)| v);
    }

    fn remove(&mut self, key: &K, min_load_factor: f32) -> Option<V> {
        if self.load_factor() < min_load_factor {
            self.resize(ResizeMode::Down);
        }

        let bucket_index = Self::bucket_index(&self._build_hasher, &key, self._buckets.len());
        let Some(bucket) = self.get_bucket_mut(bucket_index) else {
            return None;
        };

        return bucket.remove(key).map(|(_, v)| v);
    }

    fn get(&self, key: &K) -> Option<&V> {
        let bucket_index = Self::bucket_index(&self._build_hasher, &key, self._buckets.len());
        let Some(bucket) = self.get_bucket(bucket_index) else {
            return None;
        };

        return bucket.get(key);
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let bucket_index = Self::bucket_index(&self._build_hasher, &key, self._buckets.len());
        let Some(bucket) = self.get_bucket_mut(bucket_index) else {
            return None;
        };

        return bucket.get_mut(key);
    }
    
    fn get_bucket(&self, index: usize) -> Option<&KBucket<'allocator, K, V, A>> {
        if index >= self._buckets.len() {
            return None;
        }

        return Some(&self._buckets[index]);
    }

    fn get_bucket_mut(&mut self, index: usize) -> Option<&mut KBucket<'allocator, K, V, A>> {
        if index >= self._buckets.len() {
            return None;
        }

        return Some(&mut self._buckets[index]);
    }

    fn count(&self) -> usize {
        return self._buckets.len();
    }

    fn resize(&mut self, resize_mode: ResizeMode) {
        let new_buckets_len = self.calculate_buckets_count(resize_mode);
        if new_buckets_len == 0 {
            return;
        }
        
        let mut new_buckets = KVec::<'allocator, KBucket<'allocator, K, V, A>, A>::with_capacity(self._allocator, new_buckets_len);
        for _ in 0 .. new_buckets_len {
            new_buckets.push(KBucket::new(self._allocator));
        }

        while let Some(mut bucket) = self._buckets.pop() {
            while let Some(kv_maybe) = bucket._elements.pop() {
                let Some((key, value)) = kv_maybe else {
                    continue;
                };

                let bucket_index = Self::bucket_index(&self._build_hasher, &key, new_buckets_len);
                new_buckets[bucket_index].insert(key, value);
            }
        }

        self._buckets = new_buckets;
    }

    fn calculate_buckets_count(&self, resize_mode: ResizeMode) -> usize {
        return match resize_mode {
            ResizeMode::Up => {
                match self._buckets.len() {
                    0 => 2,
                    n => {
                        n.checked_mul(2)
                            .expect(kformatln!(self._allocator, "Capacity overflow, capacity: {}", n).as_str())
                    },
                }
            },
            ResizeMode::Down => {
                match self._buckets.len() {
                    0 => 0,
                    n => n.saturating_div(2),
                }
            },
        };
    }

    fn load_factor(&self) -> f32 {
        // load factor (a) = n / m;
        // n: occupied entries count
        // m: buckets count
        if self._buckets.len() == 0 {
            return 0.0;
        }
        
        let m = self._buckets.len() as f32;
        let n = self.iter().fold(0f32, |acc, bucket| acc + bucket.occupied_entries_length() as f32);
        return n / m;
    }

    fn iter(&self) -> KBucketsIterator<'_, K, V, A> {
        return KBucketsIterator::new(&self._buckets, self._buckets.len());
    }

    fn bucket_index(build_hasher: &BH, key: &K, buckets_len: usize) -> usize {
        let hash = build_hasher.hash_one(key);
        let bucket_index = hash as usize % buckets_len;
        return bucket_index;
    }
}

struct KBucket<'allocator, K : Hash + PartialEq, V : PartialEq, A : KAllocator> {
    _elements: KVec<'allocator, Option<(K, V)>, A>
}

#[allow(dead_code)]
impl<'allocator, K : Hash + PartialEq, V : PartialEq, A : KAllocator> KBucket<'allocator, K, V, A> {
    fn new(allocator: &'allocator A) -> Self {
        return Self {
            _elements: KVec::new(allocator)
        };
    }

    fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        let index_maybe = self.kv_index(&key);
        
        return match index_maybe {
            Some(index) => self._elements[index].replace((key, value)),
            None => {
                self._elements.push(Some((key, value)));
                None
            },
        };
    }

    fn remove(&mut self, key: &K) -> Option<(K, V)> {
        let index_maybe = self.kv_index(&key);
        
        return match index_maybe {
            Some(index) => self._elements.remove(index).unwrap(),
            None => None,
        };
    }

    fn get(&self, key: &K) -> Option<&V> {
        let index_maybe = self.kv_index(key);

        return match index_maybe {
            Some(index) => self._elements[index].as_ref().map(|kv| &kv.1),
            None => None,
        };
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let index_maybe = self.kv_index(key);

        return match index_maybe {
            Some(index) => self._elements[index].as_mut().map(|kv| &mut kv.1),
            None => None,
        };
    }

    fn len(&self) -> usize {
        return self._elements.len();
    }

    fn occupied_entries_length(&self) -> usize {
        return self._elements.iter().fold(0, |acc, kv_maybe| {
            acc + kv_maybe.as_ref().map(|_| 1).unwrap_or(0)
        });
    }

    fn kv_index(&self, key: &K) -> Option<usize> {
        return self._elements.iter().position(|kv_maybe| {
            return match kv_maybe {
                Some((k, _)) => k == key,
                None => false,
            };
        });
    }
}

struct KBucketsIterator<'a, K : Hash + PartialEq, V : PartialEq, A : KAllocator>  {
    _index: usize,
    _count: usize,
    _buckets: &'a KVec<'a, KBucket<'a, K, V, A>, A>
}

impl<'a, K : Hash + PartialEq, V : PartialEq, A : KAllocator> KBucketsIterator<'a, K, V, A> {
    fn new(buckets: &'a KVec<'a, KBucket<'a, K, V, A>, A>, count: usize) -> Self {
        return Self {
            _index: 0,
            _count: count,
            _buckets: buckets
        };
    }
}

impl<'a, K : Hash + PartialEq, V : PartialEq, A : KAllocator> Iterator for KBucketsIterator<'a, K, V, A> {
    type Item = &'a KBucket<'a, K, V, A>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self._index;
        if index >= self._count {
            return None;
        }

        self._index += 1;
        return Some(&self._buckets[index]);
    }
}

impl<BH: BuildHasher> KHashMapOptions<BH> {
    fn ensure_options_valid(&self) {
        assert!(self.min_load_factor > 0.0);
        assert!(self.max_load_factor > 0.0);
        assert!(self.max_load_factor > self.min_load_factor);
    }
}

impl Default for KHashMapOptions<RandomState> {
    fn default() -> Self {
        return Self {
            min_load_factor: 0.25,
            max_load_factor: 0.75,
            capacity: 0,
            build_hasher: RandomState::new(),
        }
    }
}

#[allow(unused_imports)]
mod test {
    use std::hash::{BuildHasher, Hasher, RandomState};
    use crate::{alloc::kglobal_allocator::KGlobalAllocator, collection::khashmap::KHashMap};
    use crate::collection::khashmap::{KHashMapOptions};

    #[test]
    fn khashmap_insert() {
        let allocator = KGlobalAllocator::new();
        let mut khashmap = KHashMap::<u64, u64, KGlobalAllocator, RandomState>::new(&allocator);

        khashmap.insert(1, 1);
        assert_eq!(1, *khashmap.get(&1).unwrap());
        assert_eq!(1, khashmap.length());

        khashmap.insert(123, 321);
        khashmap.insert(444, 1);
        khashmap.insert(1, 555);

        assert_eq!(321, *khashmap.get(&123).unwrap());
        assert_eq!(1, *khashmap.get(&444).unwrap());
        assert_eq!(555, *khashmap.get(&1).unwrap());

        assert_eq!(true, khashmap.get(&12345678).is_none());
        assert_eq!(3, khashmap.length());
    }

    #[test]
    fn khashmap_remove() {
        let allocator = KGlobalAllocator::new();
        let mut khashmap = KHashMap::<u64, u64, KGlobalAllocator, RandomState>::new(&allocator);

        khashmap.insert(1, 1);
        khashmap.insert(123, 321);
        khashmap.insert(444, 1);

        assert_eq!(1, khashmap.remove(&1).unwrap());
        assert_eq!(321, khashmap.remove(&123).unwrap());
        assert_eq!(1, khashmap.remove(&444).unwrap());

        assert!(khashmap.remove(&444).is_none());
        assert!(khashmap.remove(&123345).is_none());
    }

    #[cfg(not(miri))]
    #[test]
    fn khashmap_ensure_buckets_are_resized_correctly() {
        let allocator = KGlobalAllocator::new();

        let mut khashmap = KHashMap::<u64, u64, KGlobalAllocator, test_utils::hasher::IdentityBuildHasher>::with_options(
            &allocator,
            KHashMapOptions {
                min_load_factor: 0.25,
                max_load_factor: 0.75,
                capacity: 0,
                build_hasher: test_utils::hasher::IdentityBuildHasher,
            },
        );

        for value in 0..1024 {
            khashmap.insert(value, value);
        }

        for value in 0..1024 {
            assert_eq!(value, *khashmap.get(&value).unwrap());
        }

        test_utils::assert_empty_buckets(&khashmap, 1024);
        test_utils::assert_buckets_with_no_elements(&khashmap, 1024);
        assert_eq!(2048, khashmap._bucket_array._buckets.len());

        for value in 0..768 {
            khashmap.remove(&value);
        }

        test_utils::assert_empty_buckets(&khashmap, 768);
        test_utils::assert_buckets_with_no_elements(&khashmap, 768);
        assert_eq!(1024, khashmap._bucket_array._buckets.len());

        for value in 768..1024 {
            khashmap.remove(&value);
        }

        test_utils::assert_empty_buckets(&khashmap, 4);
        test_utils::assert_buckets_with_no_elements(&khashmap, 4);
        assert_eq!(4, khashmap._bucket_array._buckets.len());
    }

    #[cfg(not(miri))]
    #[test]
    fn khashmap_test_custom_load_factor() {
        let allocator = KGlobalAllocator::new();
        let mut khashmap = KHashMap::<u64, u64, KGlobalAllocator, test_utils::hasher::IdentityBuildHasher>::with_options(
            &allocator,
            KHashMapOptions {
                min_load_factor: 0.25,
                max_load_factor: 2.0,
                capacity: 0,
                build_hasher: test_utils::hasher::IdentityBuildHasher,
            },
        );

        for value in 0..1024 {
            khashmap.insert(value, value);
        }

        for value in 0..1024 {
            assert_eq!(value, *khashmap.get(&value).unwrap());
        }

        test_utils::assert_empty_buckets(&khashmap, 0);
        test_utils::assert_buckets_with_no_elements(&khashmap, 0);
        assert_eq!(512, khashmap._bucket_array._buckets.len());

        for value in 0..768 {
            khashmap.remove(&value);
        }

        test_utils::assert_empty_buckets(&khashmap, 256);
        test_utils::assert_buckets_with_no_elements(&khashmap, 256);
        assert_eq!(512, khashmap._bucket_array._buckets.len());

        for value in 768..1024 {
            khashmap.remove(&value);
        }

        test_utils::assert_empty_buckets(&khashmap, 4);
        test_utils::assert_buckets_with_no_elements(&khashmap, 4);
        assert_eq!(4, khashmap._bucket_array._buckets.len());
    }

    mod test_utils {
        use std::hash::{BuildHasher, Hasher};
        use crate::alloc::kglobal_allocator::KGlobalAllocator;
        use crate::collection::khashmap::KHashMap;
        use crate::collection::khashmap::test::test_utils;

        pub(super) mod hasher {
            use std::hash::{BuildHasher, Hasher};

            #[derive(Clone, Default)]
            pub struct IdentityBuildHasher;

            impl BuildHasher for IdentityBuildHasher {
                type Hasher = IdentityHasher;
                fn build_hasher(&self) -> Self::Hasher {
                    IdentityHasher::default()
                }
            }

            #[derive(Clone, Default)]
            pub struct IdentityHasher {
                hash: u64,
            }

            impl Hasher for IdentityHasher {
                fn finish(&self) -> u64 {
                    self.hash
                }

                fn write(&mut self, bytes: &[u8]) {
                    // For testing only — not suitable for general-purpose hashing.
                    // Just interpret up to 8 bytes as a u64.
                    let mut value = 0u64;
                    for (i, b) in bytes.iter().enumerate().take(8) {
                        value |= (*b as u64) << (i * 8);
                    }
                    self.hash = value;
                }

                fn write_u64(&mut self, v: u64) {
                    self.hash = v;
                }
            }
        }

        pub(super) fn assert_empty_buckets<BH : BuildHasher>(
            khashmap: &KHashMap<u64, u64, KGlobalAllocator, BH>,
            expected: usize
        ) {
            let mut empty_buckets = 0;

            for bucket in khashmap._bucket_array._buckets.iter() {
                if bucket._elements.is_empty() {
                    empty_buckets += 1;
                }
            }

            assert_eq!(expected, empty_buckets);
        }

        pub(super) fn assert_buckets_with_no_elements<BH : BuildHasher>(
            khashmap: &KHashMap<u64, u64, KGlobalAllocator, BH>,
            expected: usize
        ) {
            let mut empty_buckets = 0;

            for bucket in khashmap._bucket_array._buckets.iter() {
                if bucket._elements.iter().all(|element| element.is_none()) {
                    empty_buckets += 1;
                }
            }

            assert_eq!(expected, empty_buckets);
        }
    }
}
