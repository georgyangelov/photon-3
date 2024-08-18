use std::slice::Iter;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub struct VecMap<K, V> {
    values: Vec<(K, V)>
}

impl <K: Sized+PartialEq, V: Sized> VecMap<K, V> {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { values: Vec::with_capacity(capacity) }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self.get_with_index(key) {
            None => None,
            Some((_, v)) => Some(v)
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.get_with_index(&key) {
            None => self.values.push((key, value)),
            Some((i, _)) => self.values[i] = (key, value)
        }
    }

    pub fn insert_push(&mut self, key: K, value: V) {
        self.values.push((key, value))
    }

    pub fn iter(&self) -> Iter<(K, V)> {
        self.values.iter()
    }

    pub fn into_iter(self) -> IntoIter<(K, V)> {
        self.values.into_iter()
    }

    fn get_with_index(&self, key: &K) -> Option<(usize, &V)> {
        for (i, (k, v)) in self.values.iter().enumerate() {
            if k == key {
                return Some((i, v))
            }
        }

        None
    }
}