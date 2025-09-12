use std::hash::{BuildHasher, Hash, RandomState};
use crate::{alloc::kallocator::KAllocator, collection::kvec::KVec, kformatln};

const MIN_LOAD_FACTOR: f32 = 0.25;
const MAX_LOAD_FACTOR: f32 = 0.75;

enum ResizeMode {
    Up,
    Down
}

pub struct KHashMap<'a, K : Hash + PartialEq, V : PartialEq, A : KAllocator, BH : BuildHasher = RandomState> {
    _allocator: &'a A,
    _buckets: KBuckets<'a, K, V, A, BH>,
}

#[allow(dead_code)]
impl<'a, K : Hash + PartialEq, V : PartialEq, A : KAllocator> KHashMap<'a, K, V, A, RandomState> {
    pub fn new(allocator: &'a A) -> Self {
        return Self {
            _allocator: allocator,
            _buckets: KBuckets::new(allocator),
        };
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        return self._buckets.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        return self._buckets.get(key);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        return self._buckets.remove(key);
    }

    pub fn length(&self) -> usize {
        return self._buckets.iter().fold(0, |acc, bucket| acc + bucket.occupied_entiries_length());
    }
}

struct KBuckets<'a, K : Hash + PartialEq, V : PartialEq, A : KAllocator, BH : BuildHasher> {
    _allocator: &'a A,
    _buckets: KVec<'a, KBucket<'a, K, V, A>, A>,
    _count: usize,
    _build_hasher: BH
}

#[allow(dead_code)]
impl<'a, K : Hash + PartialEq, V : PartialEq, A : KAllocator> KBuckets<'a, K, V, A, RandomState> {
    fn new(allocator: &'a A) -> Self {
        return Self {
            _allocator: allocator,
            _buckets: KVec::new(allocator),
            _count: 0,
            _build_hasher: RandomState::new()
        };
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let a = self.load_factor();
        if self._buckets.is_empty() || a > MAX_LOAD_FACTOR {
            self.resize(ResizeMode::Up);
        }

        let bucket_index = Self::bucket_index(&self._build_hasher, &key, self._count);
        let Some(bucket) = self.get_bucket_mut(bucket_index) else {
            return None;
        };

        return bucket.insert(key, value).map(|(_, v)| v);
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let a = self.load_factor();
        if a < MIN_LOAD_FACTOR {
            self.resize(ResizeMode::Down);
        }

        let bucket_index = Self::bucket_index(&self._build_hasher, &key, self._count);
        let Some(bucket) = self.get_bucket_mut(bucket_index) else {
            return None;
        };

        return bucket.remove(key).map(|(_, v)| v);
    }

    fn get(&self, key: &K) -> Option<&V> {
        let bucket_index = Self::bucket_index(&self._build_hasher, &key, self._count);
        let Some(bucket) = self.get_bucket(bucket_index) else {
            return None;
        };

        return bucket.get(key);
    }

    fn get_mut(&mut self, _key: &K) -> Option<&mut V> {
        todo!()
    }
    
    fn get_bucket(&self, index: usize) -> Option<&KBucket<'a, K, V, A>> {
        if index >= self._count {
            return None;
        }

        return Some(&self._buckets[index]);
    }

    fn get_bucket_mut(&mut self, index: usize) -> Option<&mut KBucket<'a, K, V, A>> {
        if index >= self._count {
            return None;
        }

        return Some(&mut self._buckets[index]);
    }

    fn count(&self) -> usize {
        return self._count;
    }

    fn resize(&mut self, resize_mode: ResizeMode) {
        let new_buckets_len = self.calculate_buckets_count(resize_mode);
        if new_buckets_len == 0 {
            return;
        }
        
        let mut new_buckets = KVec::<'a, KBucket<'a, K, V, A>, A>::with_capacity(self._allocator, new_buckets_len);
        for _ in 0 .. new_buckets_len {
            new_buckets.push(KBucket::new(self._allocator));
        }

        while let Some(mut bucket) = self._buckets.pop() {
            while let Some(kv_maybe) = bucket._kvs.pop() {
                let Some((key, value)) = kv_maybe else {
                    continue;
                };

                let bucket_index = Self::bucket_index(&self._build_hasher, &key, new_buckets_len);
                new_buckets[bucket_index].insert(key, value);
            }
        }
        
        self._buckets = new_buckets;
        self._count = new_buckets_len;
    }

    fn calculate_buckets_count(&self, resize_mode: ResizeMode) -> usize {
        return match resize_mode {
            ResizeMode::Up => {
                match self._count {
                    0 => 2,
                    n => {
                        n.checked_mul(2)
                            .expect(kformatln!(self._allocator, "Capacity overflow, capacity: {}", n).as_str())
                    },
                }
            },
            ResizeMode::Down => {
                match self._count {
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
        if self._count == 0 {
            return 0.0;
        }
        
        let m = self._count as f32;
        let n = self.iter().fold(0f32, |acc, bucket| acc + bucket.occupied_entiries_length() as f32);
        return n / m;
    }

    fn iter<'buckets>(&'buckets self) -> KBucketsIter<'a,'buckets, K, V, A> {
        return KBucketsIter::new(&self._buckets, self._count);
    }

    fn bucket_index(build_hasher: &RandomState, key: &K, buckets_len: usize) -> usize {
        let hash = build_hasher.hash_one(key);
        let bucket_index = hash as usize % buckets_len;
        return bucket_index;
    }
}

struct KBucketsIter<'a, 'buckets, K : Hash + PartialEq, V : PartialEq, A : KAllocator>  {
    _index: usize,
    _count: usize,
    _buckets: &'buckets KVec<'a, KBucket<'a, K, V, A>, A>
}

impl<'a, 'buckets, K : Hash + PartialEq, V : PartialEq, A : KAllocator> KBucketsIter<'a, 'buckets, K, V, A> {
    fn new(buckets: &'buckets KVec<'a, KBucket<'a, K, V, A>, A>, count: usize) -> Self {
        return Self {
            _index: 0,
            _count: count,
            _buckets: buckets
        };
    }
}

impl<'a, 'buckets, K : Hash + PartialEq, V : PartialEq, A : KAllocator> Iterator for KBucketsIter<'a, 'buckets, K, V, A> {
    type Item = &'buckets KBucket<'a, K, V, A>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self._index;
        if index >= self._count {
            return None;
        }

        self._index += 1;
        return Some(&self._buckets[index]);
    }
}

struct KBucket<'a, K : Hash + PartialEq, V : PartialEq, A : KAllocator> {
    _kvs: KVec<'a, Option<(K, V)>, A>
}

#[allow(dead_code)]
impl<'a, K : Hash + PartialEq, V : PartialEq, A : KAllocator> KBucket<'a, K, V, A> {
    fn new(allocator: &'a A) -> Self {
        return Self {
            _kvs: KVec::new(allocator)
        };
    }

    fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        let index_maybe = self.kv_index(&key);
        
        return match index_maybe {
            Some(index) => self._kvs[index].replace((key, value)),
            None => {
                self._kvs.push(Some((key, value)));
                None
            },
        };
    }

    fn remove(&mut self, key: &K) -> Option<(K, V)> {
        let index_maybe = self.kv_index(&key);
        
        return match index_maybe {
            Some(index) => self._kvs.remove(index).unwrap(),
            None => None,
        };
    }

    fn get(&self, key: &K) -> Option<&V> {
        let index_maybe = self.kv_index(key);

        return match index_maybe {
            Some(index) => self._kvs[index].as_ref().map(|kv| &kv.1),
            None => None,
        };
    }

    fn occupied_entiries_length(&self) -> usize {
        return self._kvs.iter().fold(0, |acc, kv_maybe| {
            acc + kv_maybe.as_ref().map(|_| 1).unwrap_or(0)
        });
    }

    fn kv_index(&self, key: &K) -> Option<usize> {
        return self._kvs.iter().position(|kv_maybe| {
            return match kv_maybe {
                Some((k, _)) => k == key,
                None => false,
            };
        });
    }
}

#[allow(unused_imports)]
mod test {
    use crate::{alloc::global::GlobalAllocator, collection::khashmap::KHashMap};

    #[test]
    fn khashmap_insert() {
        let allocator = GlobalAllocator::new();
        let mut khashmap = KHashMap::<u64, u64, GlobalAllocator>::new(&allocator);

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
        let allocator = GlobalAllocator::new();
        let mut khashmap = KHashMap::<u64, u64, GlobalAllocator>::new(&allocator);

        khashmap.insert(1, 1);
        khashmap.insert(123, 321);
        khashmap.insert(444, 1);

        assert_eq!(1, khashmap.remove(&1).unwrap());
        assert_eq!(321, khashmap.remove(&123).unwrap());
        assert_eq!(1, khashmap.remove(&444).unwrap());
        assert!(khashmap.remove(&444).is_none());
        assert!(khashmap.remove(&123345).is_none());
    }
}
