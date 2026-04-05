use crate::hash::JacobHash;
use crate::pair::Pair;
use linked_list::LinkedList;
use std::fmt;

pub trait KeyTrait: JacobHash + Clone + PartialEq + fmt::Debug + fmt::Display {}
pub trait ValueTrait: Clone + PartialEq + fmt::Debug {}

impl<K> KeyTrait for K where
K: JacobHash + Clone + PartialEq + fmt::Debug + fmt::Display {}
impl<V> ValueTrait for V where
V: Clone + PartialEq + fmt::Debug {}


/// A HashMap implemented using chaining for collisions and a custom hash function.
pub struct HashMap<K, V> where K: KeyTrait, V: ValueTrait {
    buckets: Vec<Option<LinkedList<Pair<K, V>>>>,
    capacity: usize,
    total_pairs: usize
}

#[allow(dead_code)]
impl<K, V> HashMap<K, V> where K: KeyTrait, V: ValueTrait {
    const DEFAULT_CAPACITY: usize = 13;
    const MAX_LOAD_FACTOR: f32 = 0.5;
    const MAX_BUCKET_LENGTH: usize = 8;

    /// Creates a new HashMap.
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Creates a new HashMap with the specified initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let buckets: Vec<Option<LinkedList<Pair<K, V>>>> = vec!(None; capacity);
        let total_pairs = 0;
        Self{buckets, capacity, total_pairs}
    }

    /// Insert a key/value pair into the HashMap, replacing the existing value if there is one
    /// # Arguments
    /// * `key` the key to insert
    /// * `value` the value to insert
    pub fn insert(&mut self, key: K, value: V) {
        let index = key.jacob_hash() % self.capacity;
        let new_data = Pair::new(key, value);

        // Bucket is not empty, search for key
        if let Some(ll) = &mut self.buckets[index] {
            match ll.iter().position(|pair| pair.key == new_data.key) {
                Some(index) => {
                    ll.replace(index, new_data);
                    return
                },
                None => {
                    ll.add(new_data);
                    self.total_pairs += 1;
                    self.manage_load_factor(index);
                    return
                }
            }
        }

        // Bucket is empty, create new LL for this bucket
        let mut ll: LinkedList<Pair<K, V>> = LinkedList::new();
        ll.add(new_data);
        self.buckets[index] = Some(ll);

        // May need to resize, since we're adding a new value
        self.total_pairs += 1;
        self.manage_load_factor(index);
    }

    /// Returns an optional reference to the value associated with the given key if it exists, or
    /// `None` otherwise.
    pub fn get(&self, key: &K) -> Option<&V> {
        let index = key.jacob_hash() % self.capacity;

        // Search for key
        if let Some(ll) = &self.buckets[index] {
            let mut iter = ll.iter();
            while let Some(next) = iter.next() {
                if &next.key == key {
                    return Some(&next.value)
                }
            }
        }
        // Value doesn't exist
        None
    }

    /// Returns a reference to the value associated with the given key. Panics if it doesn't exist.
    pub fn get_unchecked(&self, key: &K) -> &V {
        let index = key.jacob_hash() % self.capacity;

        // Search for key
        if let Some(ll) = &self.buckets[index] {
            let mut iter = ll.iter();
            while let Some(next) = iter.next() {
                if &next.key == key {
                    return &next.value
                }
            }
        }
        panic!("Panicking: Key doesn't exist!")
    }

    /// Calculate and return the load factor of the HashMap (elements / bucket count).
    fn load_factor(&self) -> f32 {
        self.total_pairs as f32 / self.buckets.len() as f32
    }

    /// Check if the HashMap needs to be resized, and do it if so. Will resize if load factor is
    /// greater than 0.75, OR if the bucket at the specified index has more than 8 elements.
    /// # Arguments
    /// * `index` the index of the bucket which will have its size checked
    fn manage_load_factor(&mut self, index: usize) {
        let bucket_is_oversized = match &self.buckets[index] {
            Some(bucket) => bucket.len() > Self::MAX_BUCKET_LENGTH,
            None => false
        };
        if self.load_factor() > Self::MAX_LOAD_FACTOR || bucket_is_oversized {
            self.resize();
        }
    }

    /// Resize the HashMap to the next prime number.
    pub fn resize(&mut self) {
        self.resize_to(next_prime(self.capacity))
    }

    /// Resize the HashMap's internal storage to the specified capacity. This will rehash all
    /// existing elements.
    fn resize_to(&mut self, capacity: usize) {
        let mut new_buckets: Vec<Option<LinkedList<Pair<K, V>>>> = vec!(None; capacity);

        // Add each key/value pair to new map
        for bucket in &mut self.buckets {
            let maybe_ll = bucket.take();
            if let Some(mut ll) = maybe_ll {
                while ! ll.is_empty() {
                    let maybe_pair = ll.remove();
                    if let Some(pair) = maybe_pair {
                        let new_index = pair.key.jacob_hash() % capacity;
                        let maybe_new_ll = new_buckets[new_index].take();
                        if let Some(mut new_ll) = maybe_new_ll {
                            new_ll.add(pair);
                            new_buckets[new_index] = Some(new_ll);
                        } else {
                            let mut new_ll = LinkedList::new();
                            new_ll.add(pair);
                            new_buckets[new_index] = Some(new_ll);
                        }
                    }
                }
            }
        }
        std::mem::swap(&mut self.buckets, &mut new_buckets);
        self.capacity = capacity
    }

    /// Create an iterator over a single bucket in the HashMap.
    /// # Arguments
    /// * `maybe_bucket` the optional bucket to iterate over
    /// * `index` the starting index for iteration
    ///
    /// # Returns
    /// An optional Bucket Iterator, or `None` if the bucket doesn't exist
    fn bucket_iter(maybe_bucket: &Option<LinkedList<Pair<K, V>>>,
        index: usize) -> Option<BucketIter<K, V>> {
        if let Some(bucket) = maybe_bucket {
            return Some(BucketIter{bucket, index})
        }
        None
    }

    /// Returns an iterator over the HashMap's key/value pairs.
    pub fn iter(&self) -> Iter<K, V> {
        let buckets = &self.buckets;
        let index = 0;
        let bucket_index = 0;
        Iter{buckets, index, bucket_index}
    }

    /// Returns an iterator over the HashMap's keys.
    pub fn keys(&self) -> Keys<K, V> {
        let iter = self.iter();
        Keys{iter}
    }

    /// Returns an iterator over the HashMap's values.
    pub fn values(&self) -> Values<K, V> {
        let iter = self.iter();
        Values{iter}
    }

    /// Empty the HashMap.
    pub fn clear(&mut self) {
        std::mem::swap(self, &mut Self::new());
    }

    /// Returns the length of the HashMap.
    pub fn len(&self) -> usize {
        self.total_pairs
    }

    /// Returns a bool indicating whether or not the HashMap contains the given key.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Returns a bool indicating whether or not the HashMap is empty.
    pub fn is_empty(&self) -> bool {
        self.total_pairs == 0
    }

    /// Remove the specified key and its associated value from the HashMap.
    ///
    /// # Returns
    /// The optional value associated with the given key, or `None` if it doesn't exist
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let to_return = self.get(key).cloned();
        if let Some(_) = to_return {
            let index = key.jacob_hash() % self.capacity;
            let ll_iter = self.buckets[index].as_ref()?.iter();
            let mut new_ll: LinkedList<Pair<K, V>> = LinkedList::new();
            let _ = {
                ll_iter
                .filter(|pair| pair.key != *key)
                .map(|pair| new_ll.add(pair.clone()))
            };
            self.buckets[index] = Some(new_ll);
            self.total_pairs -= 1;
        }
        to_return
    }
    
    
    /// Returns a String representation of the HashMap.
    fn build_string(&self) -> String {
        let mut output = String::from("{");
        let iter = self.iter();
        
        for pair in iter {
            output.push_str("\n    ");
            output.push_str(&pair.to_string());
        }
        
        output.push_str("\n}");
        
        output
    }
}

impl<K, V> fmt::Display for HashMap<K, V> where K: KeyTrait, V: ValueTrait {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.build_string())
    }
}

/// An iterator for the Keys of the HashMap.
pub struct Keys<'a, K, V> where K: KeyTrait, V: ValueTrait {
    iter: Iter<'a, K, V>
}

impl<'a, K, V> Iterator for Keys<'a, K, V> where K: KeyTrait, V: ValueTrait {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.key)
    }
}

/// An iterator for the Values of the HashMap.
pub struct Values<'a, K, V> where K: KeyTrait, V: ValueTrait {
    iter: Iter<'a, K, V>
}

impl<'a, K, V> Iterator for Values<'a, K, V> where K: KeyTrait, V: ValueTrait {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.value)
    }
}

/// An iterator for a single bucket of the HashMap.
struct BucketIter<'a, K, V> where K: KeyTrait, V: ValueTrait {
    bucket: &'a LinkedList<Pair<K, V>>,
    index: usize
}

impl<'a, K, V> Iterator for BucketIter<'a, K, V> where K: KeyTrait, V: ValueTrait {
    type Item = Pair<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.bucket.iter().nth(self.index - 1).cloned()
    }
}

/// An iterator for the HashMap's Key/Value pairs.
pub struct Iter<'a, K, V> where K: KeyTrait, V: ValueTrait {
    buckets: &'a Vec<Option<LinkedList<Pair<K, V>>>>,
    index: usize,
    bucket_index: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> where K: KeyTrait, V: ValueTrait {
    type Item = Pair<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        // Early return if iterator is depleted
        if ! (self.index < self.buckets.len()) {
            return None
        }
        
        let bucket = &self.buckets[self.index];
        let mut maybe_bucket_iter = HashMap::bucket_iter(bucket, self.bucket_index);
        if let Some(bucket_iter) = &mut maybe_bucket_iter {
            match bucket_iter.next() {
                Some(pair) => {
                    self.bucket_index += 1;
                    Some(pair.clone())
                },
                None => {
                    self.index += 1;
                    self.bucket_index = 0;
                    self.next()
                }
            }
        } else {  // this bucket is empty
            self.index += 1;
            self.bucket_index = 0;
            self.next()
        }

    }
}

/// Return a bool indicating whether or not the specified number is a prime number.
fn is_prime(number: usize) -> bool {
    if number <= 1 {
        return false
    }

    for i in 2..=(number as f64).sqrt() as usize {
        if number % i == 0 {
            return false
        }
    }
    true
}

/// An array containing 20 precomputed prime numbers.
const PRIME_NUMBERS: [usize; 20] = [13, 29, 59, 127, 257, 521, 1049, 2099, 4201, 8419,
    16843, 33703, 67409, 134837, 269683, 539389, 1078787, 2157587, 4315183, 8630387];

/// Find the first prime number after the specified one.
fn next_prime(number: usize) -> usize {
    // Have to loop over pre-computed primes in O(n) - incrementing a cached index of the prime
    // array would require a mutable reference to self, which means this method can't be called
    // inside resize(). 

    // Check pre-computed primes
    let primes = &PRIME_NUMBERS;
    if number < primes[primes.len() - 1] {
        for &new_prime in primes {
            if new_prime > number {
                return new_prime;
            }
        }
    }
    // Largest pre-computed prime isn't big enough - find one manually
    let mut i = number * 2 + 1;
    while ! is_prime(i) {
        i += 1
    }
    i
}