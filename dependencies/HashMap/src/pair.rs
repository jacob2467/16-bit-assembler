use std::fmt;

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
pub struct Pair<K, V> where
        K: Clone + PartialEq + fmt::Debug + fmt::Display,
        V: Clone + PartialEq + fmt::Debug {
    pub key: K,
    pub value: V
}

impl<K, V> Pair<K, V> where
        K: Clone + PartialEq + fmt::Debug + fmt::Display,
        V: Clone + PartialEq + fmt::Debug {
    pub fn new(key: K, value: V) -> Self {
        Pair{key, value}
    }
}

impl<K, V> fmt::Display for Pair<K, V> where
        K: Clone + PartialEq + fmt::Debug + fmt::Display,
        V: Clone + PartialEq + fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.key, self.value)
    }
}