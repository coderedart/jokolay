use std::{borrow::Borrow, hash::Hash};

use derive_more::{Display, From, Into};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(transparent)]
pub struct UOMap<K: Eq + std::hash::Hash, V>(HashMap<K, V>);

#[derive(
    Debug, Clone, Serialize, Default, Deserialize, Hash, Display, derive_more::IntoIterator,
)]
#[serde(transparent)]
#[into_iterator(owned, ref)]
pub struct OMap<K: Eq + Clone + std::hash::Hash + Ord, V: Clone>(std::collections::BTreeMap<K, V>);

#[derive(
    Debug, Clone, Serialize, Default, Deserialize, Hash, Display, derive_more::IntoIterator,
)]
#[serde(transparent)]
#[into_iterator(owned, ref)]
pub struct OSet<K: Eq + Clone + std::hash::Hash + Ord>(std::collections::BTreeSet<K>);

#[derive(Debug, Clone, Serialize, Default, Deserialize, Display, derive_more::IntoIterator)]
#[serde(transparent)]
#[into_iterator(owned, ref)]
pub struct UOSet<K: Eq + Clone + std::hash::Hash>(std::collections::HashSet<K>);

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Default,
    Deserialize,
    Hash,
    From,
    Into,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
)]
#[serde(transparent)]
pub struct UTStamp(i64);

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Default,
    Deserialize,
    Hash,
    From,
    Into,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
)]
#[serde(transparent)]
pub struct CategoryID(u16);

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Default,
    Deserialize,
    Hash,
    From,
    Into,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
)]
#[serde(transparent)]
pub struct MapID(u16);

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Default,
    Deserialize,
    Hash,
    From,
    Into,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
)]
#[serde(transparent)]
pub struct MarkerID(u16);

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Default,
    Deserialize,
    Hash,
    From,
    Into,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
)]
#[serde(transparent)]
pub struct TrailID(u16);

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Default,
    Deserialize,
    Hash,
    From,
    Into,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
)]
#[serde(transparent)]
pub struct ImageID(u16);

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Default,
    Deserialize,
    Hash,
    From,
    Into,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
)]
#[serde(transparent)]
pub struct TBinID(u16);

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Default,
    Deserialize,
    Hash,
    From,
    Into,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
)]
#[serde(transparent)]
pub struct PackID(u16);


impl<'a, K: Eq + Hash, V> Default for UOMap<K, V> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
impl<'a, K: Eq + Hash, V> UOMap<K, V> {
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.0.get_mut(key)
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn iter(&'a self) -> Iter<K, V> {
        self.into_iter()
    }
    pub fn remove(&'a mut self, key: &K) -> Option<V> {
        self.0.remove(key)
    }
    pub fn iter_mut(&'a mut self) -> IterMut<'a, K, V> {
        self.into_iter()
    }
    // pub fn values(&self) -> Values<K, V>  {
    //     self.0.values()
    // }
    // pub fn values_mut(&'a mut self) -> ValuesMut<'a, K, V> {
    //     self.0.values_mut()
    // }
    // pub fn entry(&'a mut self, k: K) -> Entry<K, V> {
    //     self.0.entry(k)
    // }
}

impl<'a, K: Eq + Hash + Clone + Ord, V: Clone> OMap<K, V> {
    pub fn contains_key(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }
    pub fn new() -> Self {
        Self(std::collections::BTreeMap::new())
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.0.get_mut(key)
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn iter(&self) -> std::collections::btree_map::Iter<K, V> {
        self.0.iter()
    }
    pub fn values(&self) -> std::collections::btree_map::Values<K, V> {
        self.0.values()
    }
    pub fn values_mut(&'a mut self) -> std::collections::btree_map::ValuesMut<K, V> {
        self.0.values_mut()
    }
}
impl<K: Eq + Hash + Ord + Clone, V: Clone> From<UOMap<K, V>> for OMap<K, V>
where
    UOMap<K, V>: std::iter::IntoIterator<Item = (K, V)>,
{
    fn from(uo_map: UOMap<K, V>) -> Self {
        OMap::from_iter(uo_map)
    }
}
impl<K: Eq + Hash + Ord + Clone, V: Clone> FromIterator<(K, V)> for OMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        OMap(std::collections::BTreeMap::from_iter(iter))
    }
}
impl<K: Eq + Hash + Clone + Ord, V: Clone> FromIterator<(K, V)> for UOMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

pub struct Iter<'a, K, V>(hashbrown::hash_map::Iter<'a, K, V>);
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, K: std::hash::Hash + Eq, V> IntoIterator for &'a UOMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        Iter(self.0.iter())
    }
}
pub struct IntoIter<K, V>(hashbrown::hash_map::IntoIter<K, V>);
impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<K: Eq + Hash + Ord, V> IntoIterator for UOMap<K, V> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}
pub struct IterMut<'a, K, V>(hashbrown::hash_map::IterMut<'a, K, V>);
impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, K: std::hash::Hash + std::cmp::Eq, V> IntoIterator for &'a mut UOMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> IterMut<'a, K, V> {
        IterMut(self.0.iter_mut())
    }
}
