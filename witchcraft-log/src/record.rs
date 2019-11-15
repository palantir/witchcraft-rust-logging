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
use crate::Level;
use conjure_error::Error;
use erased_serde::Serialize;

/// Metadata of a log record.
#[derive(Clone)]
pub struct Metadata<'a> {
    level: Level,
    target: &'a str,
}

impl<'a> Metadata<'a> {
    /// Returns a builder used to create new `Metadata` values.
    #[inline]
    pub fn builder() -> MetadataBuilder<'a> {
        MetadataBuilder::new()
    }

    /// Returns the verbosity level of the metadata.
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    /// Returns the target of the metadata.
    #[inline]
    pub fn target(&self) -> &'a str {
        self.target
    }
}

/// A builder for `Metadata` values.
pub struct MetadataBuilder<'a>(Metadata<'a>);

impl<'a> Default for MetadataBuilder<'a> {
    fn default() -> MetadataBuilder<'a> {
        MetadataBuilder::new()
    }
}

impl<'a> MetadataBuilder<'a> {
    /// Creates a new `MetadataBuilder` initialized to default values.
    #[inline]
    pub fn new() -> MetadataBuilder<'a> {
        MetadataBuilder(Metadata {
            level: Level::Info,
            target: "",
        })
    }

    /// Sets the builder's verbosity level.
    ///
    /// Defaults to `Info`.
    #[inline]
    pub fn level(&mut self, level: Level) -> &mut MetadataBuilder<'a> {
        self.0.level = level;
        self
    }

    /// Sets the builder's target.
    ///
    /// Defaults to `""`.
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut MetadataBuilder<'a> {
        self.0.target = target;
        self
    }

    /// Builds a `Metadata` value.
    #[inline]
    pub fn build(&self) -> Metadata<'a> {
        self.0.clone()
    }
}

/// A log record.
#[derive(Clone)]
pub struct Record<'a> {
    metadata: Metadata<'a>,
    file: Option<&'a str>,
    line: Option<u32>,
    message: &'static str,
    safe_params: &'a [(&'static str, &'a dyn Serialize)],
    unsafe_params: &'a [(&'static str, &'a dyn Serialize)],
    error: Option<&'a Error>,
}

impl<'a> Record<'a> {
    /// Returns a `RecordBuilder` initialized to default values.
    #[inline]
    pub fn builder() -> RecordBuilder<'a> {
        RecordBuilder::new()
    }

    /// Returns the record's metadata.
    #[inline]
    pub fn metadata(&self) -> &Metadata<'a> {
        &self.metadata
    }

    /// Returns the record's verbosity level.
    #[inline]
    pub fn level(&self) -> Level {
        self.metadata.level
    }

    /// Returns the record's target.
    #[inline]
    pub fn target(&self) -> &'a str {
        self.metadata.target
    }

    /// Returns the file containing the code that created the record.
    #[inline]
    pub fn file(&self) -> Option<&'a str> {
        self.file
    }

    /// Returns the line of the code that created the record.
    #[inline]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    /// Returns the record's message.
    #[inline]
    pub fn message(&self) -> &'static str {
        self.message
    }

    /// Returns the record's safe-loggable parameters.
    #[inline]
    pub fn safe_params(&self) -> &'a [(&'static str, &dyn Serialize)] {
        self.safe_params
    }

    /// Returns the record's unsafe-loggable parameters.
    #[inline]
    pub fn unsafe_params(&self) -> &'a [(&'static str, &dyn Serialize)] {
        self.unsafe_params
    }

    /// Returns the error associated with the record.
    #[inline]
    pub fn error(&self) -> Option<&'a Error> {
        self.error
    }
}

/// A builder for `Record` values.
pub struct RecordBuilder<'a>(Record<'a>);

impl<'a> Default for RecordBuilder<'a> {
    fn default() -> RecordBuilder<'a> {
        RecordBuilder::new()
    }
}

impl<'a> RecordBuilder<'a> {
    /// Creates a `RecordBuilder` initialized to default values.
    #[inline]
    pub fn new() -> RecordBuilder<'a> {
        RecordBuilder(Record {
            metadata: Metadata::builder().build(),
            file: None,
            line: None,
            message: "",
            safe_params: &[],
            unsafe_params: &[],
            error: None,
        })
    }

    /// Sets the record's verbosity level.
    ///
    /// Defaults to `Info`.
    #[inline]
    pub fn level(&mut self, level: Level) -> &mut RecordBuilder<'a> {
        self.0.metadata.level = level;
        self
    }

    /// Sets the record's target.
    ///
    /// Defaults to `""`.
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut RecordBuilder<'a> {
        self.0.metadata.target = target;
        self
    }

    /// Sets the record's source file.
    ///
    /// Defaults to `None`.
    #[inline]
    pub fn file(&mut self, file: Option<&'a str>) -> &mut RecordBuilder<'a> {
        self.0.file = file;
        self
    }

    /// Sets the record's line.
    ///
    /// Defaults to `None`.
    #[inline]
    pub fn line(&mut self, line: Option<u32>) -> &mut RecordBuilder<'a> {
        self.0.line = line;
        self
    }

    /// Sets the record's message.
    ///
    /// Defaults to `""`.
    #[inline]
    pub fn message(&mut self, message: &'static str) -> &mut RecordBuilder<'a> {
        self.0.message = message;
        self
    }

    /// Sets the record's safe parameters.
    ///
    /// Defaults to `[]`.
    #[inline]
    pub fn safe_params(
        &mut self,
        safe_params: &'a [(&'static str, &dyn Serialize)],
    ) -> &mut RecordBuilder<'a> {
        self.0.safe_params = safe_params;
        self
    }

    /// Sets the record's unsafe parameters.
    #[inline]
    pub fn unsafe_params(
        &mut self,
        unsafe_params: &'a [(&'static str, &dyn Serialize)],
    ) -> &mut RecordBuilder<'a> {
        self.0.unsafe_params = unsafe_params;
        self
    }

    /// Sets the record's error.
    ///
    /// Defaults to `None`.
    #[inline]
    pub fn error(&mut self, error: Option<&'a Error>) -> &mut RecordBuilder<'a> {
        self.0.error = error;
        self
    }

    /// Creates a `Record`.
    #[inline]
    pub fn build(&self) -> Record<'a> {
        self.0.clone()
    }
}
