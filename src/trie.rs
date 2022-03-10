use core::iter::FromIterator;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Trie<K, V>
where
    K: core::hash::Hash + core::cmp::PartialEq + core::cmp::Eq,
{
    subtries: HashMap<K, Trie<K, V>>,
    leaf: Option<V>,
}

impl<K, V> Default for Trie<K, V>
where
    K: core::hash::Hash + core::cmp::PartialEq + core::cmp::Eq,
{
    fn default() -> Self {
        Self {
            subtries: HashMap::new(),
            leaf: None,
        }
    }
}

impl<K, V> Trie<K, V>
where
    K: core::hash::Hash + core::cmp::PartialEq + core::cmp::Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<I: IntoIterator<Item = K>>(&mut self, keys: I, value: V) -> Option<V> {
        let mut keys = keys.into_iter();
        match keys.next() {
            Some(next_key) => self
                .subtries
                .entry(next_key)
                .or_insert_with(Self::default)
                .insert(keys, value),
            None => self.leaf.replace(value),
        }
    }

    pub fn get<I: IntoIterator<Item = K>>(&self, keys: I) -> Option<&V> {
        let mut keys = keys.into_iter();
        match keys.next() {
            Some(next_key) => self.subtries.get(&next_key).and_then(|trie| trie.get(keys)),
            None => self.leaf.as_ref(),
        }
    }

impl<K, V, I> FromIterator<(I, V)> for Trie<K, V>
where
    K: core::hash::Hash + core::cmp::PartialEq + core::cmp::Eq,
    I: IntoIterator<Item = K>,
{
    fn from_iter<T: IntoIterator<Item = (I, V)>>(iter: T) -> Self {
        let mut result = Self::new();
        for (keys, item) in iter {
            result.insert(keys, item);
        }
        result
    }
}
