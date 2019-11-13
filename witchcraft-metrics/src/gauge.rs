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
