# witchcraft-rust-logging

[![CircleCI](https://circleci.com/gh/palantir/witchcraft-rust-logging.svg?style=shield)](https://circleci.com/gh/palantir/witchcraft-rust-logging)

Logging infrastructure for Witchcraft services.

## witchcraft-log

[Documentation](https://docs.rs/witchcraft-log)

A structured logging facade. It differs from the traditional [`log`](https://crates.io/crates/log) by adding a concept
of "safety" to log parameters and allowing a [`conjure-error`](https://crates.io/crates/conjure-error) `Error` to be
attached to log records.

## witchcraft-metrics

[Documentation](https://docs.rs/witchcraft-metrics)

A general-purpose metrics library. The design of the crate is based fairly closely off of the
[Dropwizard Metrics](https://github.com/dropwizard/metrics) library from the Java ecosystem.

## License

This repository is made available under the [Apache 2.0 License](http://www.apache.org/licenses/LICENSE-2.0).
