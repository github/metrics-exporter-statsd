## [v0.3.0](https://github.com/github/metrics-exporter-statsd/tree/0.3.0) - 2022-06-03

* Upgrade metrics-rs dependency to 0.19

## [v0.2.0](https://github.com/github/metrics-exporter-statsd/tree/0.2.0) - 2022-06-03

* Removed the support for installing global recorder via `StatsdBuilder`, since that can cause metrics to stop emitting when the importing app may
  link a different version of `metrics` than this library depends on.
* The calling app must now invoke `metrics::set_boxed_recorder` after building the recorder via `StatsdBuilder`
* Updated documentation to reflect that. 

## [v0.1.0](https://github.com/github/metrics-exporter-statsd/tree/0.1.0) - 2022-06-02

* Initial release.