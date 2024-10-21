## [v0.9.0](https://github.com/github/metrics-exporter-statsd/tree/0.9.0) - 2024-10-21

Version bumps for the following dependencies

- [cadence 1.5.0](https://github.com/github/metrics-exporter-statsd/pull/67)
- [thiserror 1.0.64](https://github.com/github/metrics-exporter-statsd/pull/66)
- [metrics-rs 0.24.0](https://github.com/github/metrics-exporter-statsd/pull/70)

## [v0.8.0](https://github.com/github/metrics-exporter-statsd/tree/0.7.0) - 2024-05-30

* metrics-rs [0.23]((https://github.com/github/metrics-exporter-statsd/pull/60)). This is a breaking change requiring MSRVV bump to 1.70.0 (from 1.65.0). Thanks @ijc for the PR.
* cadence   [1.4.0](https://github.com/github/metrics-exporter-statsd/pull/58)
* thiserror [1.0.59](https://github.com/github/metrics-exporter-statsd/pull/25)
  
## [v0.7.0](https://github.com/github/metrics-exporter-statsd/tree/0.7.0) - 2024-01-12

* Upgrade metrics-rs to 0.22. This is a major breaking change in `metrics`.
* Add [custom sinks](https://github.com/github/metrics-exporter-statsd/pull/23).
* [Don't panic](https://github.com/github/metrics-exporter-statsd/pull/46) when
  the application uses a `metrics` operation that is not supported by `statsd`.

## [v0.6.0](https://github.com/github/metrics-exporter-statsd/tree/0.6.0) - 2023-04-25

This release mostly contains version upgrades for the following crates:

*  metrics-rs [0.21.1](https://github.com/github/metrics-exporter-statsd/pull/29)
*  cadence    [0.29.1](https://github.com/github/metrics-exporter-statsd/pull/28)
*  thiserror  [1.0.40](https://github.com/github/metrics-exporter-statsd/pull/25)

## [v0.5.0](https://github.com/github/metrics-exporter-statsd/tree/0.5.0) - 2023-04-25

* Upgrade metrics-rs dependency to 0.21

## [v0.4.0](https://github.com/github/metrics-exporter-statsd/tree/0.4.0) - 2022-07-27

* Upgrade metrics-rs dependency to 0.20
* Update describe_* methods to use SharedString to comply with new version requirements.

## [v0.3.0](https://github.com/github/metrics-exporter-statsd/tree/0.3.0) - 2022-06-03

* Upgrade metrics-rs dependency to 0.19

## [v0.2.0](https://github.com/github/metrics-exporter-statsd/tree/0.2.0) - 2022-06-03

* Removed the support for installing global recorder via `StatsdBuilder`, since that can cause metrics to stop emitting when the importing app may
  link a different version of `metrics` than this library depends on.
* The calling app must now invoke `metrics::set_boxed_recorder` after building the recorder via `StatsdBuilder`
* Updated documentation to reflect that. 

## [v0.1.0](https://github.com/github/metrics-exporter-statsd/tree/0.1.0) - 2022-06-02

* Initial release.
