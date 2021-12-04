// I don't want a dependency on indexmap (it unconditionally depends on rayon???), so this will have
// to do instead
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrderedMap<K, V>(pub Vec<(K, V)>);

impl<K, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K: Eq, V> OrderedMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.0
            .iter()
            .find(|entry| &entry.0 == k)
            .map(|entry| &entry.1)
    }

    pub fn insert(&mut self, k: K, v: V) {
        match self.0.iter_mut().find(|entry| entry.0 == k) {
            Some(entry) => entry.1 = v,
            None => self.0.push((k, v)),
        }
    }

    pub fn get_or_insert_with(&mut self, k: K, v: impl FnOnce() -> V) -> &mut V {
        match self.0.iter().position(|entry| entry.0 == k) {
            Some(i) => &mut self.0[i].1,
            None => {
                self.0.push((k, v()));
                &mut self.0.last_mut().expect("we just inserted").1
            }
        }
    }
}

impl<K, V> IntoIterator for OrderedMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
