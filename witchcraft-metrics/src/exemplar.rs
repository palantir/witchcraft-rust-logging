// Copyright 2024 Palantir Technologies, Inc.
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

use std::any::TypeId;
use std::sync::Arc;

mod private {
    pub struct PrivacyToken;
}

/// Extra contextual data associated with individual metric measurements.
///
/// Exemplars can store arbitrary data, so this type is opaque and only exposes APIs to downcast a `dyn Exemplar` trait
/// object to its underlying concrete type. It is automatically implemented for all `'static + Sync + Send` types.
pub trait Exemplar: 'static + Sync + Send {
    #[doc(hidden)]
    fn __private_api_type_id(&self, _: private::PrivacyToken) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl dyn Exemplar {
    /// Returns `true` if the exemplar value's type is `T`.
    pub fn is<T>(&self) -> bool
    where
        T: Exemplar,
    {
        self.__private_api_type_id(private::PrivacyToken) == TypeId::of::<T>()
    }

    /// Attempts to downcast the exemplar's value to the type `T` if it has that type.
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Exemplar,
    {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn Exemplar as *const T)) }
        } else {
            None
        }
    }

    /// Attempts to downcast the exemplar's value to the type `T` if it has that type.
    pub fn downcast_arc<T>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>>
    where
        T: Exemplar,
    {
        if self.is::<T>() {
            unsafe { Ok(Arc::from_raw(Arc::into_raw(self).cast::<T>())) }
        } else {
            Err(self)
        }
    }
}

impl<T> Exemplar for T where T: 'static + Sync + Send {}
