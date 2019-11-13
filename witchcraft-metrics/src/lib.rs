//! A general-purpose metrics library.
//!
//! The design of the crate is based fairly closely off of the [Dropwizard Metrics] library from the Java ecosystem.
//!
//! # Examples
//!
//! ```
//! use witchcraft_metrics::{MetricRegistry, MetricId, Metric};
//! use std::time::Duration;
//!
//! // A `MetricRegistry` stores metrics.
//! let registry = MetricRegistry::new();
//!
//! // Metrics are identified by an ID, which consists of a name and set of "tags"
//! let yaks_shaved = registry.counter(MetricId::new("shavings").with_tag("animal", "yak"));
//! // You can also pass a string directly for metric IDs that don't have tags
//! let request_timer = registry.timer("server.requests");
//!
//! // do some work and record some values.
//! for yak in find_some_yaks() {
//!     shave_yak(yak);
//!     yaks_shaved.inc();
//! }
//!
//! // Grab a snapshot of the metrics currently registered and print their values:
//! for (id, metric) in &registry.metrics() {
//!     match metric {
//!         Metric::Counter(counter) => println!("{:?} is a counter with value {}", id, counter.count()),
//!         Metric::Timer(timer) => {
//!             let nanos = timer.snapshot().value(0.99);
//!             let duration = Duration::from_nanos(nanos as u64);
//!             println!("{:?} is a timer with 99th percentile {:?}", id, duration);
//!         }
//!         _ => {}
//!     }
//! }
//!
//! # fn find_some_yaks() -> &'static [()] { &[] }
//! # fn shave_yak(_: &()) {}
//! ```
//!
//! [Dropwizard Metrics]: https://github.com/dropwizard/metrics
#![warn(missing_docs)]

pub use crate::counter::*;
pub use crate::gauge::*;
pub use crate::histogram::*;
pub use crate::meter::*;
pub use crate::metric_id::*;
pub use crate::registry::*;
pub use crate::reservoir::*;
pub use crate::timer::*;

mod counter;
mod gauge;
mod histogram;
mod meter;
mod metric_id;
mod registry;
mod reservoir;
mod timer;
