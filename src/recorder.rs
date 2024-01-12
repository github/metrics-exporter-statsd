use std::sync::Arc;
use std::time::Duration;

use cadence::{Counted, Distributed, Gauged, Histogrammed, MetricBuilder, StatsdClient, Timed};
use metrics::{Counter, CounterFn, SharedString};
use metrics::{Gauge, GaugeFn};
use metrics::{Histogram, HistogramFn};
use metrics::{Key, KeyName, Label, Metadata, Recorder, Unit};

use crate::types::HistogramType;

/// A recorder for sending the reported metrics to Statsd.
/// Under the hood this recorder uses [`StatsdClient`] implementation provided by [`cadence`] crate.
/// The calls to `register_*` methods are pretty much ignored because statsd doesn't have any facility
/// for registering metrics with descriptions. This recorder's main responsibility is to map metrics
/// library's interface/types to a supported [`StatsdClient`] calls/types.
pub struct StatsdRecorder {
    statsd: Arc<StatsdClient>,
    default_histogram: HistogramType,
}

impl StatsdRecorder {
    /// Initialize [`StatsdRecorder`] with provided [`cadence::StatsdClient`].
    pub fn new(statsd: StatsdClient, default_histogram: HistogramType) -> Self {
        StatsdRecorder {
            statsd: Arc::new(statsd),
            default_histogram,
        }
    }
}

impl Recorder for StatsdRecorder {
    fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // statsd recording does not support descriptions.
    }

    fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // statsd recording does not support descriptions.
    }

    fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // statsd recording does not support descriptions.
    }

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        Counter::from_arc(Arc::new(Handle::new(
            key.clone(),
            self.statsd.clone(),
            self.default_histogram,
        )))
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        Gauge::from_arc(Arc::new(Handle::new(
            key.clone(),
            self.statsd.clone(),
            self.default_histogram,
        )))
    }

    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        Histogram::from_arc(Arc::new(Handle::new(
            key.clone(),
            self.statsd.clone(),
            self.default_histogram,
        )))
    }
}

struct Handle {
    key: Key,
    statsd: Arc<StatsdClient>,
    default_histogram: HistogramType,
}

impl Handle {
    fn new(key: Key, statsd: Arc<StatsdClient>, default_histogram: HistogramType) -> Self {
        Handle {
            key,
            statsd,
            default_histogram,
        }
    }

    fn apply_tags<'m, 'c, M>(
        labels: Vec<&'m Label>,
        mb: MetricBuilder<'m, 'c, M>,
    ) -> MetricBuilder<'m, 'c, M>
    where
        M: cadence::Metric + From<String>,
    {
        labels
            .into_iter()
            .fold(mb, |acc, l| acc.with_tag(l.key(), l.value()))
    }
}

impl CounterFn for Handle {
    fn increment(&self, value: u64) {
        // this is an unfortunate conversion, probably deserves an issue on cadence?
        let mb = self.statsd.count_with_tags(self.key.name(), value as i64);
        Self::apply_tags(self.key.labels().collect(), mb).send();
    }

    fn absolute(&self, _value: u64) {
        // statsd recording does not support setting absolute values on counters
    }
}

impl GaugeFn for Handle {
    fn increment(&self, _value: f64) {
        // statsd recording does not support incrementing gauge values because it doesn't know the
        // prior value.
    }

    fn decrement(&self, _value: f64) {
        // statsd recording does not support decrementing gauge values because it doesn't know the
        // prior value.
    }

    fn set(&self, value: f64) {
        let mb = self.statsd.gauge_with_tags(self.key.name(), value);
        Self::apply_tags(self.key.labels().collect(), mb).send();
    }
}

impl HistogramFn for Handle {
    fn record(&self, value: f64) {
        let (hist_type, labels) = HistogramType::type_from(&self.key);
        match hist_type.unwrap_or(self.default_histogram) {
            HistogramType::Distribution => {
                let mb = self.statsd.distribution_with_tags(self.key.name(), value);
                Self::apply_tags(labels, mb).send();
            }
            HistogramType::Timer => {
                // Cadence expects the timer to be in milliseconds and metrics lib reports those as seconds
                // we translate the seconds to milliseconds. Unfortunately there's a downcase involved here
                // from u128 to u64.
                let time_in_ms = Duration::from_secs_f64(value).as_millis() as u64;
                let mb = self.statsd.time_with_tags(self.key.name(), time_in_ms);
                Self::apply_tags(labels, mb).send();
            }
            HistogramType::Histogram => {
                let mb = self.statsd.histogram_with_tags(self.key.name(), value);
                Self::apply_tags(labels, mb).send();
            }
        };
    }
}
