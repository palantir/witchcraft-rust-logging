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

static EMPTY: Lazy<Arc<HashMap<&'static str, Any>>> = Lazy::new(|| Arc::new(HashMap::new()));

thread_local! {
    static MDC: RefCell<Snapshot> = RefCell::new(Snapshot {
        safe_mdc: EMPTY.clone(),
        unsafe_mdc: EMPTY.clone(),
    });
}

/// Clears the contents of the MDC.
pub fn clear() {
    MDC.with(|v| {
        let mut mdc = v.borrow_mut();
        // try to preserve capacity if we're the unique owner
        match Arc::get_mut(&mut mdc.safe_mdc) {
            Some(safe_mdc) => safe_mdc.clear(),
            None => mdc.safe_mdc = EMPTY.clone(),
        }
        match Arc::get_mut(&mut mdc.unsafe_mdc) {
            Some(unsafe_mdc) => unsafe_mdc.clear(),
            None => mdc.unsafe_mdc = EMPTY.clone(),
        }
    });
}

/// Inserts a new safe parameter into the MDC.
///
/// # Panics
///
/// Panics if the value cannot be serialized into an [`Any`].
pub fn insert_safe<T>(key: &'static str, value: T)
where
    T: Serialize,
{
    MDC.with(|v| {
        Arc::make_mut(&mut v.borrow_mut().safe_mdc)
            .insert(key, Any::new(value).expect("value failed to serialize"))
    });
}

/// Inserts a new unsafe parameter into the MDC.
///
/// # Panics
///
/// Panics if the value cannot be serialized into an [`Any`].
pub fn insert_unsafe<T>(key: &'static str, value: T)
where
    T: Serialize,
{
    MDC.with(|v| {
        Arc::make_mut(&mut v.borrow_mut().unsafe_mdc)
            .insert(key, Any::new(value).expect("value failed to serialize"))
    });
}

/// Removes the specified safe parameter from the MDC.
pub fn remove_safe(key: &'static str) {
    MDC.with(|v| Arc::make_mut(&mut v.borrow_mut().safe_mdc).remove(key));
}

/// Removes the specified unsafe parameter from the MDC.
pub fn remove_unsafe(key: &'static str) {
    MDC.with(|v| Arc::make_mut(&mut v.borrow_mut().unsafe_mdc).remove(key));
}

/// Takes a snapshot of the MDC.
pub fn snapshot() -> Snapshot {
    MDC.with(|v| {
        let mdc = v.borrow();
        Snapshot {
            safe_mdc: mdc.safe_mdc.clone(),
            unsafe_mdc: mdc.unsafe_mdc.clone(),
        }
    })
}

/// Overwrites the MDC with a snapshot.
pub fn set(snapshot: Snapshot) {
    MDC.with(|v| *v.borrow_mut() = snapshot);
}

/// Swaps the MDC with a snapshot, returning a guard object which will un-swap them when it drops.
///
/// Changes to the MDC while the guard is live will be reflected in the snapshot when the guard drops.
pub fn with(snapshot: &mut Snapshot) -> WithGuard<'_> {
    MDC.with(|v| mem::swap(&mut *v.borrow_mut(), snapshot));
    WithGuard { snapshot }
}

/// A portable snapshot of the MDC.
pub struct Snapshot {
    safe_mdc: Arc<HashMap<&'static str, Any>>,
    unsafe_mdc: Arc<HashMap<&'static str, Any>>,
}

impl Snapshot {
    /// Returns an iterator over the safe parameters in the snapshot.
    #[inline]
    pub fn safe_iter(&self) -> Iter<'_> {
        Iter {
            it: self.safe_mdc.iter(),
        }
    }

    /// Returns an iterator over the unsafe parameters in the snapshot.
    #[inline]
    pub fn unsafe_iter(&self) -> Iter<'_> {
        Iter {
            it: self.unsafe_mdc.iter(),
        }
    }
}

/// An iterator over parameters in a snapshot.
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

/// The guard type returned by [`with`].
pub struct WithGuard<'a> {
    snapshot: &'a mut Snapshot,
}

impl Drop for WithGuard<'_> {
    fn drop(&mut self) {
        MDC.with(|v| mem::swap(&mut *v.borrow_mut(), self.snapshot));
    }
}
