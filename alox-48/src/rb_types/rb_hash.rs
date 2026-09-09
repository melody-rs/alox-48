#![allow(missing_docs)]

use crate::{
    Continue, Deserialize, HashDefaultAccess, HashKeyAccess, Serialize, SerializeHashDefault,
    SerializeHashKey, Value, Visitor,
};
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RbHash {
    pub map: ValueMap,
    pub default: Option<Box<Value>>,
}

pub type ValueMap = IndexMap<Value, Value>;

impl Default for RbHash {
    fn default() -> Self {
        Self {
            map: IndexMap::new(),
            default: None,
        }
    }
}

impl RbHash {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(n: usize) -> Self {
        Self {
            map: IndexMap::with_capacity(n),
            default: None,
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn contains_key(&self, key: &Value) -> bool {
        self.map.contains_key(key)
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        self.map.get(key).or(self.default.as_deref())
    }

    pub fn get_mut(&mut self, key: &Value) -> Option<&mut Value> {
        self.map.get_mut(key).or(self.default.as_deref_mut())
    }

    pub fn insert(&mut self, key: Value, value: Value) -> Option<Value> {
        self.map.insert(key, value)
    }

    pub fn remove(&mut self, key: &Value) -> Option<Value> {
        self.map.swap_remove(key)
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, Value, Value> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> indexmap::map::IterMut<'_, Value, Value> {
        self.map.iter_mut()
    }

    pub fn keys(&self) -> indexmap::map::Keys<'_, Value, Value> {
        self.map.keys()
    }

    pub fn values(&self) -> indexmap::map::Values<'_, Value, Value> {
        self.map.values()
    }

    pub fn values_mut(&mut self) -> indexmap::map::ValuesMut<'_, Value, Value> {
        self.map.values_mut()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
}

impl AsRef<ValueMap> for RbHash {
    fn as_ref(&self) -> &ValueMap {
        &self.map
    }
}

impl From<RbHash> for ValueMap {
    fn from(value: RbHash) -> Self {
        value.map
    }
}

impl From<ValueMap> for RbHash {
    fn from(value: ValueMap) -> Self {
        Self {
            map: value,
            default: None,
        }
    }
}

impl IntoIterator for RbHash {
    type Item = <ValueMap as IntoIterator>::Item;
    type IntoIter = <ValueMap as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <ValueMap as IntoIterator>::into_iter(self.map)
    }
}

impl<'a> IntoIterator for &'a RbHash {
    type Item = <&'a ValueMap as IntoIterator>::Item;
    type IntoIter = <&'a ValueMap as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <&ValueMap as IntoIterator>::into_iter(&self.map)
    }
}

impl<'a> IntoIterator for &'a mut RbHash {
    type Item = <&'a mut ValueMap as IntoIterator>::Item;
    type IntoIter = <&'a mut ValueMap as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <&mut ValueMap as IntoIterator>::into_iter(&mut self.map)
    }
}

pub(crate) struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = RbHash;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a ruby hash")
    }

    fn visit_hash<A>(self, mut current: Continue<A, ()>) -> crate::DeResult<Self::Value>
    where
        A: HashKeyAccess<'de, Finished = ()>,
    {
        let mut hash = RbHash::with_capacity(current.len());

        while let Continue::Next(next) = current {
            let (k, v, next) = next.next_entry()?;
            hash.insert(k, v);
            current = next;
        }

        Ok(hash)
    }

    fn visit_hash_default<A>(
        self,
        mut current: Continue<A, A::Finished>,
    ) -> crate::DeResult<Self::Value>
    where
        A: HashKeyAccess<'de>,
        A::Finished: HashDefaultAccess<'de>,
    {
        let mut hash = RbHash::with_capacity(current.len());

        loop {
            match current {
                Continue::Next(next) => {
                    let (k, v, next) = next.next_entry()?;
                    hash.insert(k, v);
                    current = next;
                }
                Continue::Finished(default) => {
                    hash.default = Some(default.deserialize_default()?);
                    break Ok(hash);
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for RbHash {
    fn deserialize<D>(deserializer: D) -> crate::DeResult<Self>
    where
        D: crate::DeserializerTrait<'de>,
    {
        deserializer.deserialize(HashVisitor)
    }
}

fn serialize_with_default<S: crate::SerializerTrait>(
    serializer: S,
    map: &ValueMap,
    default: &Value,
) -> crate::SerResult<S::Ok> {
    let mut current = serializer.serialize_hash_default(map.len())?;
    let mut iter = map.iter();

    loop {
        match current {
            Continue::Next(next) => {
                let (k, v) = iter.next().expect("should be a next value");
                current = next.serialize_entry(k, v)?;
            }
            Continue::Finished(d) => break d.serialize_default(default),
        }
    }
}

fn serialize<S: crate::SerializerTrait>(serializer: S, map: &ValueMap) -> crate::SerResult<S::Ok> {
    let mut current = serializer.serialize_hash(map.len())?;
    let mut iter = map.iter();

    loop {
        match current {
            Continue::Next(next) => {
                let (k, v) = iter.next().expect("should be a next value");
                current = next.serialize_entry(k, v)?;
            }
            Continue::Finished(v) => break Ok(v),
        }
    }
}

impl Serialize for RbHash {
    fn serialize<S>(&self, serializer: S) -> crate::SerResult<S::Ok>
    where
        S: crate::SerializerTrait,
    {
        match self.default.as_deref() {
            Some(default) => serialize_with_default(serializer, &self.map, default),
            None => serialize(serializer, &self.map),
        }
    }
}
