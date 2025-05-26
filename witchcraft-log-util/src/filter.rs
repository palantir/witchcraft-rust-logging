// Copyright 2025 Palantir Technologies, Inc.
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
//! A prefix-based target filter.

use sequence_trie::SequenceTrie;
use witchcraft_log::{LevelFilter, Metadata};

/// A prefix-based target filter.
///
/// The filter is configured with a top-level [`LevelFilter`] and additional per-target filters. Targets are interpreted
/// as a hierarchy by splitting on `::`. For example a target `foo::bar` will have a filter for the `foo` target
/// applied to it if there is not also a filter for `foo::bar` itself.
pub struct Filter {
    trie: SequenceTrie<String, LevelFilter>,
}

impl Filter {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> Builder {
        Builder {
            filter: Filter {
                trie: SequenceTrie::new(),
            },
            root: LevelFilter::Error,
        }
    }

    /// Determines if the provided log metadata matches the filter.
    pub fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level()
            <= *self
                .trie
                .get_ancestor(metadata.target().split("::"))
                .unwrap()
    }

    /// Returns the most verbose level in the filter.
    pub fn max_level(&self) -> LevelFilter {
        self.trie.values().max().copied().unwrap()
    }
}

/// A builder for [`Filter`]s.
pub struct Builder {
    filter: Filter,
    root: LevelFilter,
}

impl Builder {
    /// Sets the level used for targets that don't match a more specific directive.
    ///
    /// Defaults to [`LevelFilter::Error`].
    #[inline]
    pub fn level(mut self, level: LevelFilter) -> Self {
        self.root = level;
        self
    }

    /// Sets the level used for a specific target.
    #[inline]
    pub fn target_level(mut self, target: &str, level: LevelFilter) -> Self {
        self.filter.trie.insert(target.split("::"), level);
        self
    }

    /// Consumes the builder, returning a filter.
    #[inline]
    pub fn build(mut self) -> Filter {
        self.filter.trie.insert_owned([], self.root);
        self.filter
    }
}

#[cfg(test)]
mod test {
    use witchcraft_log::Level;

    use super::*;

    #[test]
    fn empty() {
        let filter = Filter::builder().build();

        assert!(
            filter.enabled(
                &Metadata::builder()
                    .level(Level::Error)
                    .target("foo")
                    .build()
            )
        );

        assert!(!filter.enabled(&Metadata::builder().level(Level::Warn).target("foo").build()));
    }

    #[test]
    fn nonempty() {
        let filter = Filter::builder()
            .level(LevelFilter::Warn)
            .target_level("foo", LevelFilter::Debug)
            .target_level("foo::bar", LevelFilter::Off)
            .build();

        assert!(
            filter.enabled(
                &Metadata::builder()
                    .level(Level::Error)
                    .target("bar")
                    .build()
            )
        );
        assert!(!filter.enabled(&Metadata::builder().level(Level::Info).target("bar").build()));

        assert!(filter.enabled(&Metadata::builder().level(Level::Info).target("foo").build()));
        assert!(
            !filter.enabled(
                &Metadata::builder()
                    .level(Level::Trace)
                    .target("foo")
                    .build()
            )
        );

        assert!(
            !filter.enabled(
                &Metadata::builder()
                    .level(Level::Fatal)
                    .target("foo::bar")
                    .build()
            )
        );

        assert!(
            filter.enabled(
                &Metadata::builder()
                    .level(Level::Fatal)
                    .target("foo::buz")
                    .build()
            )
        );
    }
}
