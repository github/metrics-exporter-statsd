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
