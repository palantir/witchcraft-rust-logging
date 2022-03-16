// Copyright 2021 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//! A Mapped Diagnostic Context (MDC) for Witchcraft loggers.
//!
//! An MDC is a thread local map containing extra parameters. Witchcraft logging implementations should include the
//! contents of the MDC in service logs.
use conjure_object::Any;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{hash_map, HashMap};
use std::mem;
use std::sync::Arc;

static EMPTY: Lazy<Map> = Lazy::new(|| Map {
    map: Arc::new(HashMap::new()),
});

thread_local! {
    static MDC: RefCell<Snapshot> = RefCell::new(Snapshot::new());
}

/// Inserts a new safe parameter into the MDC.
///
/// # Panics
///
/// Panics if the value cannot be serialized into an [`Any`].
pub fn insert_safe<T>(key: &'static str, value: T) -> Option<Any>
where
    T: Serialize,
{
    MDC.with(|v| v.borrow_mut().safe_mut().insert(key, value))
}

/// Inserts a new unsafe parameter into the MDC.
///
/// # Panics
///
/// Panics if the value cannot be serialized into an [`Any`].
pub fn insert_unsafe<T>(key: &'static str, value: T) -> Option<Any>
where
    T: Serialize,
{
    MDC.with(|v| v.borrow_mut().unsafe_mut().insert(key, value))
}

/// Removes the specified safe parameter from the MDC.
pub fn remove_safe(key: &str) -> Option<Any> {
    MDC.with(|v| v.borrow_mut().safe_mut().remove(key))
}

/// Removes the specified unsafe parameter from the MDC.
pub fn remove_unsafe(key: &str) -> Option<Any> {
    MDC.with(|v| v.borrow_mut().unsafe_mut().remove(key))
}

/// Takes a snapshot of the MDC.
///
/// The snapshot and MDC are not connected - updates to the snapshot will not affect the MDC and vice versa.
pub fn snapshot() -> Snapshot {
    MDC.with(|v| v.borrow().clone())
}

/// Clears the contents of the MDC.
pub fn clear() {
    MDC.with(|v| {
        let mut mdc = v.borrow_mut();
        mdc.safe_mut().clear();
        mdc.unsafe_mut().clear();
    });
}

/// Overwrites the MDC with a snapshot, returning the previous state.
pub fn set(snapshot: Snapshot) -> Snapshot {
    MDC.with(|v| mem::replace(&mut *v.borrow_mut(), snapshot))
}

/// Swaps the MDC with a snapshot in-place.
pub fn swap(snapshot: &mut Snapshot) {
    MDC.with(|v| mem::swap(&mut *v.borrow_mut(), snapshot));
}

/// A map of MDC entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Map {
    map: Arc<HashMap<&'static str, Any>>,
}

impl Default for Map {
    #[inline]
    fn default() -> Self {
        EMPTY.clone()
    }
}

impl Map {
    /// Returns a new, empty map.
    #[inline]
    pub fn new() -> Self {
        Map::default()
    }

    /// Removes all entries from the map.
    #[inline]
    pub fn clear(&mut self) {
        // try to preserve capacity if we're the unique owner
        match Arc::get_mut(&mut self.map) {
            Some(map) => map.clear(),
            None => *self = Map::new(),
        }
    }

    /// Returns the number of entries in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Determines if the map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Looks up a value in the map.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&Any> {
        self.map.get(key)
    }

    /// Determines if the map contains the specified key.
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// Inserts a new entry into the map, returning the old value corresponding to the key.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot be serialized into an [`Any`].
    #[inline]
    pub fn insert<V>(&mut self, key: &'static str, value: V) -> Option<Any>
    where
        V: Serialize,
    {
        let value = Any::new(value).expect("value failed to serialize");
        Arc::make_mut(&mut self.map).insert(key, value)
    }

    /// Removes an entry from the map, returning its value.
    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<Any> {
        Arc::make_mut(&mut self.map).remove(key)
    }

    /// Returns an iterator over the entries in the map.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            it: self.map.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a Map {
    type Item = (&'static str, &'a Any);

    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the entries in a [`Map`].
pub struct Iter<'a> {
    it: hash_map::Iter<'a, &'static str, Any>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'static str, &'a Any);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|(k, v)| (*k, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

impl ExactSizeIterator for Iter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.it.len()
    }
}

/// A portable snapshot of the MDC.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Snapshot {
    safe: Map,
    unsafe_: Map,
}

impl Snapshot {
    /// Returns a new, empty snapshot.
    #[inline]
    pub fn new() -> Self {
        Snapshot::default()
    }

    /// Returns a shared reference to the safe entries in the snapshot.
    #[inline]
    pub fn safe(&self) -> &Map {
        &self.safe
    }

    /// Returns a mutable reference to the safe entries in the snapshot.
    #[inline]
    pub fn safe_mut(&mut self) -> &mut Map {
        &mut self.safe
    }

    /// Returns a shared reference to the unsafe entries in the snapshot.
    #[inline]
    pub fn unsafe_(&self) -> &Map {
        &self.unsafe_
    }

    /// Returns a shared reference to the unsafe entries in the snapshot.
    #[inline]
    pub fn unsafe_mut(&mut self) -> &mut Map {
        &mut self.unsafe_
    }
}
