# metrics-exporter-statsd

A [metrics-rs](https://github.com/metrics-rs/metrics/) exporter that supports reporting metrics to StatsD. This exporter is basically a thin wrapper on top of the [`cadence`] crate which supports Statsd/Datadog style metrics.

Check out [crates.io](https://crates.io/crates/metrics-exporter-statsd) documentation for how to use this library. 

## Contribution

This project welcomes contributions. See CONTRIBUTING.md for details on how to build, install, and contribute.

```
cargo build test

```
## License 

This project is licensed under the terms of the MIT open source license. Please refer to [MIT](./LICENSE.md) for the full terms.

## Acknowledgement

metrics-exporter-statsd is authored, reviewed and tested by the code search team at GitHub:

 - @gorzell
 - @look
 - @mbellani 
 - @tclem
 - @terrhorn
 - @colinwm
 - @jorendorff
 - @aneubeck

Special thanks to the authors of [metrics-rs](https://github.com/metrics-rs/metrics/) and [cadence](https://github.com/56quarters/cadence/) libraries. 