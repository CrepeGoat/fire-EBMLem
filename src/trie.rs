use std::collections::HashMap;

#[derive(Debug)]
pub struct Trie<K, V>
where
    K: core::hash::Hash + core::cmp::PartialEq + core::cmp::Eq,
{
    subtrie: HashMap<K, Trie<K, V>>,
    leaf: Option<V>,
}

impl<K, V> Default for Trie<K, V>
where
    K: core::hash::Hash + core::cmp::PartialEq + core::cmp::Eq,
{
    fn default() -> Self {
        Self {
            subtrie: HashMap::new(),
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

    pub fn insert<I: Iterator<Item = K>>(&mut self, mut keys: I, value: V) -> Option<V> {
        match keys.next() {
            Some(next_key) => self
                .subtrie
                .entry(next_key)
                .or_insert_with(Self::default)
                .insert(keys, value),
            None => self.leaf.replace(value),
        }
    }

    pub fn get<I: Iterator<Item = K>>(&mut self, mut keys: I, value: V) -> Option<&V> {
        todo!()
    }
}
