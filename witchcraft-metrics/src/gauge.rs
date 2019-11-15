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
use serde::Serialize;
use serde_value::Value;

/// A generalized metric which computes an arbitrary value.
///
/// It is implemented for all closures returning serializable types.
pub trait Gauge: 'static + Sync + Send {
    /// Returns the serialized value.
    fn value(&self) -> Value;
}

impl<F, R> Gauge for F
where
    F: Fn() -> R + 'static + Sync + Send,
    R: Serialize,
{
    fn value(&self) -> Value {
        serde_value::to_value(self()).expect("value failed to serialize")
    }
}
