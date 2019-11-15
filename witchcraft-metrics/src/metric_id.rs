// Copyright 2019 Palantir Technologies, Inc.
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
use std::borrow::Cow;
use std::collections::{btree_map, BTreeMap};

/// An identifier of a metric.
///
/// It consists of a name as well as a set of tags that can be used to further filter and partition the metrics.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MetricId {
    name: Cow<'static, str>,
    tags: Tags,
}

impl MetricId {
    /// Creates a new metric ID with the specified name and no tags.
    pub fn new<T>(name: T) -> MetricId
    where
        T: Into<Cow<'static, str>>,
    {
        MetricId {
            name: name.into(),
            tags: Tags(BTreeMap::new()),
        }
    }

    /// A builder-style method adding a tag to the metric ID.
    pub fn with_tag<K, V>(mut self, key: K, value: V) -> MetricId
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        self.tags.0.insert(key.into(), value.into());
        self
    }

    /// Returns the ID's name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the ID's tags.
    #[inline]
    pub fn tags(&self) -> &Tags {
        &self.tags
    }
}

impl From<String> for MetricId {
    #[inline]
    fn from(s: String) -> Self {
        MetricId::new(s)
    }
}

impl From<&'static str> for MetricId {
    #[inline]
    fn from(s: &'static str) -> Self {
        MetricId::new(s)
    }
}

impl From<Cow<'static, str>> for MetricId {
    #[inline]
    fn from(s: Cow<'static, str>) -> MetricId {
        MetricId::new(s)
    }
}

/// The tags associated with a metric ID.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Tags(BTreeMap<Cow<'static, str>, Cow<'static, str>>);

impl Tags {
    /// Returns an iterator over the tags.
    #[inline]
    pub fn iter(&self) -> TagsIter<'_> {
        TagsIter(self.0.iter())
    }
}

impl<'a> IntoIterator for &'a Tags {
    type Item = (&'a str, &'a str);
    type IntoIter = TagsIter<'a>;

    #[inline]
    fn into_iter(self) -> TagsIter<'a> {
        self.iter()
    }
}

/// An iterator over the key-value pairs of a metric ID's tags.
pub struct TagsIter<'a>(btree_map::Iter<'a, Cow<'static, str>, Cow<'static, str>>);

impl<'a> Iterator for TagsIter<'a> {
    type Item = (&'a str, &'a str);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (&**k, &**v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> ExactSizeIterator for TagsIter<'a> {}
