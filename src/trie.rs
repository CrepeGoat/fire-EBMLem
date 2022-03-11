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

    pub fn iter(&self) -> impl core::iter::Iterator<Item = (Vec<&K>, &V)> {
        let mut trie_buffer = vec![(0_usize, None, self)];

        let iter_subtrie_meta = core::iter::from_fn(move || {
            let next_item = trie_buffer.pop();
            if let Some((depth, key, trie)) = next_item {
                trie_buffer.extend(trie.subtries.iter().map(|(k, v)| (depth + 1, Some(k), v)));
            }
            next_item
        });
        iter_subtrie_meta
            .scan(Vec::new(), move |keypath, (depth, key, trie)| {
                keypath.truncate(depth.saturating_sub(1)); // the first item has depth = 0, all others have depth > 0
                keypath.extend(key.iter()); // the first key value is None, all others are Some(k)
                Some((keypath.clone(), trie))
            })
            .filter_map(|(keypath, trie)| trie.leaf.as_ref().map(|value| (keypath, value)))
    }

    pub fn iter_values(&self) -> impl core::iter::Iterator<Item = &V> {
        let mut trie_buffer = vec![self];

        core::iter::from_fn(move || {
            if let Some(trie) = trie_buffer.pop() {
                trie_buffer.extend(trie.subtries.values());
                Some(trie)
            } else {
                None
            }
        })
        .filter_map(|trie| trie.leaf.as_ref())
    }

    pub fn iter_sub<I: IntoIterator<Item = K>>(
        &self,
        keys: I,
    ) -> Option<std::collections::hash_map::Iter<'_, K, Trie<K, V>>> {
        let mut keys = keys.into_iter();
        match keys.next() {
            Some(next_key) => self
                .subtries
                .get(&next_key)
                .and_then(|trie| trie.iter_sub(keys)),
            None => Some(self.subtries.iter()),
        }
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
