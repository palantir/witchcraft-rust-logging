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
use std::any::TypeId;
use std::sync::Arc;

mod private {
    pub struct PrivacyToken;
}

/// A generalized metric which computes an arbitrary value.
///
/// It is implemented for all closures returning serializable types.
pub trait Gauge: 'static + Sync + Send {
    /// Returns the serialized value.
    fn value(&self) -> Value;

    // PrivacyToken can't be named outside of this crate, so it prevents anyone from overriding this default
    // implementation in another crate. That allows us to trust it to be correct in the downcast methods below.
    #[doc(hidden)]
    fn __private_api_type_id(&self, _: private::PrivacyToken) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl dyn Gauge {
    /// Returns `true` if the gauge value's type is `T`.
    pub fn is<T>(&self) -> bool
    where
        T: Gauge,
    {
        self.__private_api_type_id(private::PrivacyToken) == TypeId::of::<T>()
    }

    /// Attempts to downcast the gauge's value to the type `T` if it has that type.
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Gauge,
    {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn Gauge as *const T)) }
        } else {
            None
        }
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
