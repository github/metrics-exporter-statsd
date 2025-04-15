//! A [`metrics`] exporter that supports reporting metrics to Statsd. This exporter is basically
//! a thin wrapper on top of the [`cadence`] crate which supports Statsd/Datadog style metrics.
//!
//! # Warnings
//!
//! * **Versions of this crate are tightly coupled to metrics crate versions.**
//!
//! * [`metrics::Counter::absolute`], [`metrics::Gauge::increment`], and
//!   [`metrics::Gauge::decrement`] are not supported. Statsd doesn't have these concepts.
//!   Unfortunately this means that if the application is using these methods, the metrics will
//!   silently be missing or wrong.
//!
//! # Usage
//!
//! ```
//! use metrics_exporter_statsd::StatsdBuilder;
//!
//! let recorder = StatsdBuilder::from("127.0.0.1", 8125)
//! .with_queue_size(5000)
//! .with_buffer_size(1024)
//! .build(Some("prefix"))
//! .expect("Could not create StatsdRecorder");
//!
//! metrics::set_global_recorder(recorder);
//! ```
//!
//! You can then continue to use [`metrics`] as usual:
//!
//! ```
//! metrics::counter!("counter.name").increment(1);
//! ```
//!
//! Labels are translated to datadog style tags:
//!
//!```
//! metrics::gauge!("gauge.name", "tag" => "value").set(100.0);
//!```
//! will translate to `gauge.name:50.25|g|#tag:value` and should render appropriately in systems
//! like Datadog.
//!
//! # Queue Size and Buffer Size
//!
//! The supplied queue size and buffer size are used to construct the two different
//! [`MetricSink`](cadence::MetricSink)s provided by [`cadence`]:
//!
//! 1.  [`BufferedUdpMetricSink`](cadence::BufferedUdpMetricSink) controls how much data
//!     should be buffered before actually flushing it over the network/socket. By default this value
//!     is conservatively set to `256` and should be tuned based on the client needs/experience.
//!
//! 2.  [`QueuingMetricSink`](cadence::QueuingMetricSink) controls how many elements should be
//!     allowed to queue when the demand on `StatsdClient` is high, this value is currently
//!     defauled to `5000`. This value should also be tuned according to the client
//!     needs/experience. It's important to note that once the queue is full the `StatsdClient`
//!     will error out and overflow metrics will not be reported to statsd.
//!
//! As documented in `cadence`'s documentation, this is the preferred way to configure `cadence`
//! in production. This interface doesn't allow you to configure an unbounded queue, you must provide
//! a queue size or one is chosen for you.
//!
//! # Histograms
//! The default behavior if you do not specify a global preference, or an explict tag is to send
//! `histogram!` metrics as Histograms.  If you do set an alternative global preference but would
//! like to override it for a given metric you can still do so:
//!
//! ```
//! metrics::histogram!("metric.name", "histogram"=> "histogram","tag"=>"value").record(100.0)
//! ```
//! This will emit the usual histogram metric this `metric.name:100|h|#tag:value`.
//!
//!# Distributions
//! Some implementations of StatsD like Dogstatsd support the concept of distributions, that
//! aggregate the measurments on the server instead of the agent. This allows for more accurate
//! calculation percentiles by systems like Datadog.
//!
//! Unfortunately  the metrics library doesn't have the direct interface for reporting distributions
//! e.g. `metrics::distribution!("...")` (which is understandable as it may not be broadly applicable).
//!
//! This library works around that by morphing `metrics::histogram!` into distribution if you provide
//! provide an appropriate hint label.
//!
//! **Reporting distributions:**
//! ```
//! metrics::histogram!("metric.name", "histogram"=>"distribution", "tag"=>"value").record(100.0)
//! ```
//! This will emit a metric like this: `metric.name:100|d|#tag:value`, note the metric type has
//! changed from `h` to `d`.
//!
//! # Timers
//! StatsD specification does have the concept of timers that more or less behave like histograms e.g.
//! they are aggregated at the agent, support for timer metrics is similar to distribution.
//!
//! **Reporting timers:**
//! ```
//! metrics::histogram!("metric.name", "histogram"=>"timer", "tag"=>"value").record(100.0)
//! ```
//! This will emit a metric like this: `metric.name:100|ms|#tag:value`, note the metric type has
//! changed from `h` to `ms`.
//!
//! # Chaging the default type of histogram
//!
//! If your application mostly is interested in distribution or timers, you can indicate that to
//! `StatsdBuilder` in the following way:
//!
//! ```
//! use metrics_exporter_statsd::StatsdBuilder;
//!
//! let recorder = StatsdBuilder::from("127.0.0.1", 8125)
//! .with_queue_size(5000)
//! .with_buffer_size(1024)
//! .histogram_is_distribution()
//! .build(Some("prefix"))
//! .expect("Could not create StatsdRecorder");
//!
//! metrics::set_global_recorder(recorder);
//!```
//!
//! Once the exporter is marked this way then all the histograms will be reported as distributions
//! by default unless labeled differently. For example following statement:
//!
//! ```
//! metrics::histogram!("metric.name", "tag"=>"value").record(100.0)
//! ```
//! This will emit a metric like this: `metric.name:100|d|#tag:value`, note the metric type has
//! emitted here is `d` and not `h`.
//!
//! **Note:** Most of the other metrics-rs builders provide a convenience method for installing a global recorder. E.g
//! for Prometheus or TCP metrics exporters you could do something along the lines of `PrometheusBuilder::new().install()`.
//!
//! This library does not have an `.install()` method. Instead, use `.build()` and call
//! `metrics::set_boxed_recorder`, as in the example code. This ensures that if you ever have a version mismatch
//! between `metrics-recorder-statsd` and `metrics`, you'll get a build-time error (rather than Cargo silently
//! linking in two versions of `metrics`, which would result in `metrics` silently dropping all your data).
mod recorder;

pub use self::recorder::*;

mod builder;
mod types;

pub use self::builder::*;
