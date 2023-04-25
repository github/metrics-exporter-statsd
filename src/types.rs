use cadence::Metric;
use metrics::{Key, Label};

pub struct Histogram {
    repr: String,
}

impl Metric for Histogram {
    fn as_metric_str(&self) -> &str {
        self.repr.as_str()
    }
}

/// This enum represents all the different histogram transformations that we support. Each histogram
/// value also takes tags which should be remaining tags after stripping of the `histogram` label.
#[derive(Clone, Copy)]
pub enum HistogramType {
    Distribution,
    Timer,
    Histogram,
}

impl HistogramType {
    const HISTOGRAM_HINT: &'static str = "histogram";
    // Returns the type of histogram from the provided label, it also filters out the
    // type hint from the labels so that it doesn't end up in the reporting system.
    pub(crate) fn type_from(key: &Key) -> (Option<HistogramType>, Vec<&Label>) {
        let (hist_label, labels): (Vec<&Label>, Vec<&Label>) =
            key.labels().partition(|l| l.key() == Self::HISTOGRAM_HINT);

        let hist_type = hist_label.first().map(|l| HistogramType::from(l.value()));

        (hist_type, labels)
    }
}

impl From<&str> for HistogramType {
    fn from(hist_type: &str) -> Self {
        match hist_type {
            "timer" => HistogramType::Timer,
            "distribution" => HistogramType::Distribution,
            _ => HistogramType::Histogram,
        }
    }
}
