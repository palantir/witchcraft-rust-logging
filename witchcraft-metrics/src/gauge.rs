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
use std::any::Any;
use std::sync::Arc;

/// A generalized metric which computes an arbitrary value.
///
/// It is implemented for all closures returning serializable types.
pub trait Gauge: Any + Sync + Send {
    /// Returns the serialized value.
    fn value(&self) -> Value;
}

impl dyn Gauge {
    /// Returns `true` if the gauge value's type is `T`.
    pub fn is<T>(&self) -> bool
    where
        T: Gauge,
    {
        (self as &dyn Any).is::<T>()
    }

    /// Attempts to downcast the gauge's value to the type `T` if it has that type.
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Gauge,
    {
        (self as &dyn Any).downcast_ref()
    }

    /// Attempts to downcast the gauge's value to the type `T` if it has that type.
    pub fn downcast_arc<T>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>>
    where
        T: Gauge,
    {
        if self.is::<T>() {
            unsafe { Ok(Arc::from_raw(Arc::into_raw(self).cast::<T>())) }
        } else {
            Err(self)
        }
    }
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

#[cfg(test)]
mod test {
    use super::*;

    struct TestGauge {
        value: i64,
    }

    impl Gauge for TestGauge {
        fn value(&self) -> Value {
            Value::I64(self.value)
        }
    }

    #[test]
    fn downcast() {
        let gauge: Arc<dyn Gauge> = Arc::new(TestGauge { value: 42 });

        assert!(gauge.is::<TestGauge>());
        assert!(!gauge.is::<fn() -> Value>());

        assert!(gauge.downcast_ref::<fn() -> Value>().is_none());
        assert_eq!(gauge.downcast_ref::<TestGauge>().unwrap().value, 42);
        assert!(gauge.clone().downcast_arc::<fn() -> Value>().is_err());
        assert_eq!(gauge.downcast_arc::<TestGauge>().ok().unwrap().value, 42);
    }
}
