// Copyright 2026 Palantir Technologies, Inc.
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

//! Utilities for Witchcraft metric logs.

use conjure_object::Utc;
use conjure_object::log_safety::AssertLogSafe;
use witchcraft_logging_api::objects::{MetricLogV1, metric_log_v1};
use witchcraft_metrics::{Counter, Gauge, Histogram, Meter, Metric, MetricId, Timer};

const NANOS_PER_MICRO: i64 = 1_000;
const NANOS_PER_MICRO_F64: f64 = NANOS_PER_MICRO as f64;

/// Converts a metric into a `MetricLogV1`, evaluating gauges inline.
pub fn from_metric(id: &MetricId, metric: &Metric) -> MetricLogV1 {
    match metric {
        Metric::Counter(m) => from_counter(id, m),
        Metric::Meter(m) => from_meter(id, m),
        Metric::Gauge(m) => from_gauge(id, &**m),
        Metric::Histogram(m) => from_histogram(id, m),
        Metric::Timer(m) => from_timer(id, m),
    }
}

/// Converts a counter into a `MetricLogV1`.
pub fn from_counter(id: &MetricId, counter: &Counter) -> MetricLogV1 {
    finish(
        id,
        builder(id)
            .metric_type("counter")
            .insert_values("count", counter.count()),
    )
}

/// Converts a meter into a `MetricLogV1`.
pub fn from_meter(id: &MetricId, meter: &Meter) -> MetricLogV1 {
    finish(
        id,
        builder(id)
            .metric_type("meter")
            .insert_values("count", meter.count())
            .insert_values("1m", meter.one_minute_rate()),
    )
}

/// Converts a gauge into a `MetricLogV1`, evaluating it on the calling thread.
pub fn from_gauge(id: &MetricId, gauge: &dyn Gauge) -> MetricLogV1 {
    finish(
        id,
        builder(id)
            .metric_type("gauge")
            .insert_values("value", gauge.value()),
    )
}

/// Converts a histogram into a `MetricLogV1`.
pub fn from_histogram(id: &MetricId, histogram: &Histogram) -> MetricLogV1 {
    let snapshot = histogram.snapshot();
    finish(
        id,
        builder(id)
            .metric_type("histogram")
            .insert_values("max", snapshot.max())
            .insert_values("p95", snapshot.value(0.95))
            .insert_values("p99", snapshot.value(0.99))
            .insert_values("p999", snapshot.value(0.999))
            .insert_values("count", histogram.count()),
    )
}

/// Converts a timer into a `MetricLogV1`, reporting durations in microseconds.
pub fn from_timer(id: &MetricId, timer: &Timer) -> MetricLogV1 {
    let snapshot = timer.snapshot();
    finish(
        id,
        builder(id)
            .metric_type("timer")
            .insert_values("max", snapshot.max() / NANOS_PER_MICRO)
            .insert_values("p95", snapshot.value(0.95) / NANOS_PER_MICRO_F64)
            .insert_values("p99", snapshot.value(0.99) / NANOS_PER_MICRO_F64)
            .insert_values("p999", snapshot.value(0.999) / NANOS_PER_MICRO_F64)
            .insert_values("count", timer.count())
            .insert_values("1m", timer.one_minute_rate()),
    )
}

/// Formats a metric ID as `name[key:value,...]` for safe logging.
pub fn safe_metric_name_with_tags(id: &MetricId) -> AssertLogSafe<String> {
    let mut name = id.name().to_string();

    if id.tags().iter().next().is_some() {
        name.push('[');
        let mut first = true;
        for (key, value) in id.tags() {
            if !first {
                name.push(',');
            }
            first = false;
            name.push_str(key);
            name.push(':');
            name.push_str(value);
        }
        name.push(']');
    }

    AssertLogSafe(name)
}

fn builder(id: &MetricId) -> metric_log_v1::Builder<metric_log_v1::MetricTypeStage> {
    MetricLogV1::builder()
        .type_("metric.1")
        .time(Utc::now())
        .metric_name(id.name())
}

fn finish(id: &MetricId, builder: metric_log_v1::Builder<metric_log_v1::Complete>) -> MetricLogV1 {
    builder
        .tags(
            id.tags()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        )
        .build()
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::{Value, json};
    use std::time::Duration;

    fn values(log: &MetricLogV1) -> Value {
        serde_json::to_value(log.values()).unwrap()
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn counter() {
        let id = MetricId::new("foo").with_tag("a", "b");
        let counter = Counter::default();
        counter.add(3);

        let log = from_counter(&id, &counter);
        assert_eq!(log.type_(), "metric.1");
        assert_eq!(log.metric_name(), "foo");
        assert_eq!(log.metric_type(), "counter");
        assert_eq!(log.tags().get("a").map(String::as_str), Some("b"));
        assert_eq!(values(&log), json!({"count": 3}));
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn meter() {
        let meter = Meter::default();
        meter.mark(2);

        let log = from_meter(&MetricId::new("foo"), &meter);
        assert_eq!(log.metric_type(), "meter");
        assert_eq!(values(&log)["count"], json!(2));
        assert!(values(&log).get("1m").is_some());
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn gauge() {
        let log = from_gauge(&MetricId::new("foo"), &|| 7);
        assert_eq!(log.metric_type(), "gauge");
        assert_eq!(values(&log), json!({"value": 7}));
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn histogram() {
        let histogram = Histogram::default();
        histogram.update(5);

        let log = from_histogram(&MetricId::new("foo"), &histogram);
        assert_eq!(log.metric_type(), "histogram");
        assert_eq!(
            values(&log),
            json!({"max": 5, "p95": 5., "p99": 5., "p999": 5., "count": 1}),
        );
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn timer() {
        let timer = Timer::default();
        timer.update(Duration::from_micros(5));

        let log = from_timer(&MetricId::new("foo"), &timer);
        assert_eq!(log.metric_type(), "timer");
        let values = values(&log);
        assert_eq!(values["max"], json!(5));
        assert_eq!(values["p99"], json!(5.));
        assert_eq!(values["count"], json!(1));
        assert!(values.get("1m").is_some());
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn from_metric_dispatches() {
        let gauge: std::sync::Arc<dyn Gauge> = std::sync::Arc::new(|| 1);
        let log = from_metric(&MetricId::new("foo"), &Metric::Gauge(gauge));
        assert_eq!(log.metric_type(), "gauge");
    }

    #[test]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn name() {
        assert_eq!(safe_metric_name_with_tags(&MetricId::new("foo")).0, "foo");
        assert_eq!(
            safe_metric_name_with_tags(&MetricId::new("foo").with_tag("a", "b").with_tag("c", "d"))
                .0,
            "foo[a:b,c:d]",
        );
    }
}
