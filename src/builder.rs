use std::net::UdpSocket;

use cadence::{BufferedUdpMetricSink, QueuingMetricSink, StatsdClient};
use metrics::SetRecorderError;

use crate::recorder::StatsdRecorder;
use crate::types::HistogramType;
use thiserror::Error;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8125;
const DEFAULT_QUEUE_SIZE: usize = 5000;
const DEFAULT_BUFFER_SIZE: usize = 256;
const CLIENT_UDP_HOST: &str = "0.0.0.0";

#[derive(Error, Debug)]
pub enum StatsdError {
    /// This error indicates that the caller has supplied an invalid/empty host name.
    #[error("Empty hostname is not allowed")]
    InvalidHost,

    /// The caller specified port 0. In TCP/UDP programming generally, this is sometimes used to
    /// tell the system "pick a port for me", but we don't support it.
    #[error("Port number must be nonzero")]
    InvalidPortZero,

    /// MetricError indicates that there was an error reporting metrics to statsd, this is directly
    /// mapped from [`cadence::MetricError`].
    #[error("Metrics reporting error")]
    MetricError {
        #[from]
        source: cadence::MetricError,
    },

    /// Any I/O-related errors, e.g. UDP connection/bind errors.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// An error indicating that there was a problem registering [`StatsdRecorder`] with the
    /// [`metrics-rs`] system.
    #[error("Could not register the metrics recorder")]
    RecorderError {
        #[from]
        source: SetRecorderError<StatsdRecorder>,
    },
}

/// [`StatsdBuilder`] is responsible building and configuring a [`StatsdRecorder`].
pub struct StatsdBuilder {
    host: String,
    port: u16,
    queue_size: Option<usize>,
    buffer_size: Option<usize>,
    default_histogram: HistogramType,
    client_udp_host: String,
    default_tags: Vec<(String, String)>,
}

impl StatsdBuilder {
    /// Configures the [`StatsdBuilder`] with provided host and port number. A [`StatsdError`]
    /// is returned to the caller if the values supplied for host and/or port are invalid.
    /// You can further customize other variables like `queue_size` and `buffer_size` by calling
    /// appropriate `with_*` methods on the builder.
    pub fn from<S: Into<String>>(host: S, port: u16) -> Self {
        StatsdBuilder {
            host: host.into(),
            port,
            queue_size: None,
            buffer_size: None,
            default_histogram: HistogramType::Histogram,
            client_udp_host: CLIENT_UDP_HOST.to_string(),
            default_tags: Vec::new(),
        }
    }

    /// Configure queue size for this builder, the queue size is eventually passed down to the
    /// underlying StatsdClient to control how many elements should be allowed to buffer in a queue.
    /// The default value for the queue size is `5000`, Statsd client will error out and drop the
    /// new elements being sent to it once it hits this capacity.
    pub fn with_queue_size(mut self, queue_size: usize) -> Self {
        self.queue_size = Some(queue_size);
        self
    }

    /// Buffer size controls how much should be buffered in StatsdClient's memory before they are
    /// actually written out over the socket. This value is conservatively set to 256 bytes and
    /// should be adjusted according to the application needs.
    pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = Some(buffer_size);
        self
    }

    /// Host address to which the local udp socket would be bound, this address defaults to
    /// `0.0.0.0`. Be careful with using `127.0.0.1` as systems like kubernetes might blackhole
    /// all the traffic routed to that address.
    pub fn with_client_udp_host<S: Into<String>>(mut self, client_udp_host: S) -> Self {
        self.client_udp_host = client_udp_host.into();
        self
    }

    /// A hint for the metric emitter to determine how the histogram metrics should be emitted,
    /// all the histogram metrics will be sent as distribution when running in this mode unless
    /// specified otherwise via a label.
    pub fn histogram_is_distribution(mut self) -> Self {
        self.default_histogram = HistogramType::Distribution;
        self
    }

    /// A hint for the metric emitter to determine how the histogram metrics should be emitted,
    /// all the histogram metrics will be sent as timer when running in this mode unless specified
    /// otherwise via a label.
    pub fn histogram_is_timer(mut self) -> Self {
        self.default_histogram = HistogramType::Timer;
        self
    }

    /// Add a default tag with key and value to all statsd metrics produced with this recorder.
    pub fn with_default_tag<K, V>(mut self, key: K, value: V) -> Self
    where
        K: ToString,
        V: ToString,
    {
        self.default_tags.push((key.to_string(), value.to_string()));
        self
    }

    /// This method is responsible building the StatsdRecorder. It configures the underlying metrics sink for
    /// the [`StatsdClient`] with the values provided e.g. `queue_size`, `buffer_size` etc.
    ///
    /// All the metrics emitted from the recorder are prefixed with the prefix that's provided here.
    ///
    /// # Examples
    /// ```
    /// use metrics_exporter_statsd::StatsdBuilder;
    /// let recorder = StatsdBuilder::from("localhost", 8125)
    ///                .build(Some("prefix"))
    ///                .expect("Could not create StatsdRecorder");
    ///
    /// metrics::set_global_recorder(recorder);
    /// metrics::counter!("counter.name").increment(10);
    /// ```
    /// will emit a counter metric name as `prefix.counter.name`
    pub fn build(self, prefix: Option<&str>) -> Result<StatsdRecorder, StatsdError> {
        self.is_valid()?;
        // create a local udp socket where the communication needs to happen, the port is set to
        // 0 so that we can pick any available port on the host. We also want this socket to be
        // non-blocking
        let socket = UdpSocket::bind(format!("{}:{}", self.client_udp_host, 0))?;
        socket.set_nonblocking(true)?;
        // Initialize the statsd client with metrics sink that will be used to collect and send
        // the metrics to the remote host.
        let host = (self.host, self.port);
        // Initialize buffered udp metrics sink with the provided or default capacity, this allows
        // statsd client (cadence) to buffer metrics upto the configured size in memory before, flushing
        // to network.
        let udp_sink = BufferedUdpMetricSink::with_capacity(
            host,
            socket,
            self.buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE),
        )?;
        // Initialize a bounded QueuingMetricSink so that we are not buffering unlimited items onto
        // statsd client's queue, statsd client will error out when the queue is full.
        let sink = QueuingMetricSink::with_capacity(
            udp_sink,
            self.queue_size.unwrap_or(DEFAULT_BUFFER_SIZE),
        );

        let mut builder = StatsdClient::builder(prefix.unwrap_or(""), sink);
        for (key, value) in self.default_tags {
            builder = builder.with_tag(key, value);
        }

        Ok(StatsdRecorder::new(builder.build(), self.default_histogram))
    }

    fn is_valid(&self) -> Result<(), StatsdError> {
        if self.host.trim().is_empty() {
            return Err(StatsdError::InvalidHost);
        }
        if self.port == 0 {
            return Err(StatsdError::InvalidPortZero);
        }
        Ok(())
    }
}

impl Default for StatsdBuilder {
    fn default() -> Self {
        StatsdBuilder {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            queue_size: Some(DEFAULT_QUEUE_SIZE),
            buffer_size: Some(DEFAULT_BUFFER_SIZE),
            default_histogram: HistogramType::Histogram,
            client_udp_host: CLIENT_UDP_HOST.to_string(),
            default_tags: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use std::time::Duration;

    use metrics::{Key, Label, Recorder};

    use crate::builder::StatsdBuilder;
    use crate::recorder::StatsdRecorder;

    pub struct Environ {
        server_socket: UdpSocket,
        recorder: StatsdRecorder,
    }

    impl Environ {
        fn setup() -> (UdpSocket, StatsdBuilder) {
            let server_socket = UdpSocket::bind("127.0.0.1:0")
                .expect("localhost should always be a valid socket address");
            server_socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("failed to set the read timeout on our localhost socket");
            let port = server_socket
                .local_addr()
                .expect("socket should have a local addr")
                .port();

            let builder = StatsdBuilder::from("127.0.0.1", port)
                .with_queue_size(1)
                .with_buffer_size(10);
            (server_socket, builder)
        }

        pub fn new(prefix: Option<&str>) -> Self {
            let (server_socket, builder) = Environ::setup();
            let recorder = builder
                .build(prefix)
                .expect("test env should build a valid recorder");
            Environ {
                server_socket,
                recorder,
            }
        }

        pub fn new_histogram_is_distribution() -> Self {
            let (server_socket, builder) = Environ::setup();
            let recorder = builder
                .histogram_is_distribution()
                .build(None)
                .expect("test env should build a valid recorder");
            Environ {
                server_socket,
                recorder,
            }
        }

        pub fn new_histogram_is_timer() -> Self {
            let (server_socket, builder) = Environ::setup();
            let recorder = builder
                .histogram_is_timer()
                .build(None)
                .expect("test env should build a valid recorder");
            Environ {
                server_socket,
                recorder,
            }
        }

        fn receive_on_server(&self) -> String {
            let mut buff = [0; 100];

            let size = self
                .server_socket
                .recv(&mut buff)
                .expect("could not receive on server socket");
            let data = &buff[..size];
            let request = std::str::from_utf8(data).expect("request is no a valid UTF-8 string");
            String::from(request)
        }
    }

    static METADATA: metrics::Metadata =
        metrics::Metadata::new(module_path!(), metrics::Level::INFO, Some(module_path!()));

    #[test]
    #[should_panic]
    fn bad_host_name() {
        StatsdBuilder::from("", 10)
            .build(None)
            .expect("this should panic");
    }

    #[test]
    #[should_panic]
    fn bad_port() {
        StatsdBuilder::from("127.0.0.1", 0)
            .build(None)
            .expect("this should panic");
    }

    #[test]
    fn counter() {
        let env = Environ::new(None);
        let key = Key::from_name("counter.name");
        let counter = env.recorder.register_counter(&key, &METADATA);
        counter.increment(1);
        assert_eq!("counter.name:1|c", env.receive_on_server());
    }

    #[test]
    fn counter_with_tags() {
        let env = Environ::new(None);
        let tags = vec![Label::new("t1", "v1"), Label::new("t2", "v2")];
        let key = Key::from(("counter.name", tags));

        let coutner = env.recorder.register_counter(&key, &METADATA);
        coutner.increment(10);
        assert_eq!("counter.name:10|c|#t1:v1,t2:v2", env.receive_on_server());
    }

    #[test]
    fn gauge() {
        let env = Environ::new(None);
        let key = Key::from_name("gauge.name");
        let gauge = env.recorder.register_gauge(&key, &METADATA);
        gauge.set(50.25);
        assert_eq!("gauge.name:50.25|g", env.receive_on_server());
    }

    #[test]
    fn gauge_with_tags() {
        let env = Environ::new(None);
        let tags = vec![Label::new("t1", "v1"), Label::new("t2", "v2")];
        let key = Key::from(("gauge.name", tags));
        let gauge = env.recorder.register_gauge(&key, &METADATA);
        gauge.set(50.25);
        assert_eq!("gauge.name:50.25|g|#t1:v1,t2:v2", env.receive_on_server());
    }

    #[test]
    fn histogram() {
        let env = Environ::new(None);
        let key = Key::from_name("histogram.name");
        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.00);
        assert_eq!("histogram.name:100|h", env.receive_on_server());
    }

    #[test]
    fn histogram_with_decimals() {
        let env = Environ::new(None);
        let key = Key::from_name("histogram.name");
        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.52);
        assert_eq!("histogram.name:100.52|h", env.receive_on_server());
    }

    #[test]
    fn distribution_with_decimals() {
        let env = Environ::new_histogram_is_distribution();
        let key = Key::from_name("distribution.name");

        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.52);
        assert_eq!("distribution.name:100.52|d", env.receive_on_server());
    }

    #[test]
    fn histogram_with_tags() {
        let env = Environ::new(None);
        let tags = vec![Label::new("t1", "v1"), Label::new("t2", "v2")];
        let key = Key::from(("histogram.name", tags));

        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.00);
        assert_eq!("histogram.name:100|h|#t1:v1,t2:v2", env.receive_on_server());
    }

    #[test]
    fn histogram_as_distribution() {
        let env = Environ::new(None);
        let tags = vec![
            Label::new("t1", "v1"),
            Label::new("t2", "v2"),
            Label::new("histogram", "distribution"),
        ];
        let key = Key::from(("distribution.name", tags));

        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.00);
        assert_eq!(
            "distribution.name:100|d|#t1:v1,t2:v2",
            env.receive_on_server()
        );
    }

    #[test]
    fn distribution_with_prefix() {
        let env = Environ::new(Some("blackbird"));
        let tags = vec![
            Label::new("t1", "v1"),
            Label::new("t2", "v2"),
            Label::new("histogram", "distribution"),
        ];
        let key = Key::from(("distribution.name", tags));

        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.00);
        assert_eq!(
            "blackbird.distribution.name:100|d|#t1:v1,t2:v2",
            env.receive_on_server()
        );
    }

    #[test]
    fn histogram_with_prefix() {
        let env = Environ::new(Some("blackbird"));
        let tags = vec![Label::new("t1", "v1"), Label::new("t2", "v2")];
        let key = Key::from(("histogram.name", tags));

        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.00);
        assert_eq!(
            "blackbird.histogram.name:100|h|#t1:v1,t2:v2",
            env.receive_on_server()
        );
    }

    #[test]
    fn histogram_as_timer() {
        let env = Environ::new(None);
        let tags = vec![
            Label::new("t1", "v1"),
            Label::new("t2", "v2"),
            Label::new("histogram", "timer"),
        ];
        let key = Key::from(("histogram.name", tags));

        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.00);
        // metrics-rs reports the unit as seconds and we convert it to ms
        assert_eq!(
            "histogram.name:100000|ms|#t1:v1,t2:v2",
            env.receive_on_server()
        );
    }

    #[test]
    fn default_histogram_to_distribution() {
        let env = Environ::new_histogram_is_distribution();
        let tags = vec![Label::new("t1", "v1"), Label::new("t2", "v2")];
        let key = Key::from(("histogram.name", tags));

        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.00);
        assert_eq!("histogram.name:100|d|#t1:v1,t2:v2", env.receive_on_server());
    }

    #[test]
    fn default_histogram_to_timer() {
        let env = Environ::new_histogram_is_timer();
        let tags = vec![Label::new("t1", "v1"), Label::new("t2", "v2")];
        let key = Key::from(("histogram.name", tags));

        let histogram = env.recorder.register_histogram(&key, &METADATA);
        histogram.record(100.00);
        // metrics-rs reports the unit as seconds and we convert it to ms
        assert_eq!(
            "histogram.name:100000|ms|#t1:v1,t2:v2",
            env.receive_on_server()
        );
    }

    #[test]
    fn prefix() {
        let env = Environ::new(Some("koelbird"));
        let key = Key::from_name("counter.name");
        let counter = env.recorder.register_counter(&key, &METADATA);
        counter.increment(1);
        assert_eq!("koelbird.counter.name:1|c", env.receive_on_server());
    }

    #[test]
    fn test_default_tags() {
        let (server_socket, builder) = Environ::setup();
        let recorder = builder
            .with_default_tag("app_name", "test")
            .with_default_tag("blackbird_cluster", "magenta")
            .build(None)
            .expect("test env should build a valid recorder");
        let env = Environ {
            server_socket,
            recorder,
        };

        let key = Key::from_name("counter.name");
        let counter = env.recorder.register_counter(&key, &METADATA);

        counter.increment(1);
        assert_eq!(
            "counter.name:1|c|#app_name:test,blackbird_cluster:magenta",
            env.receive_on_server()
        );
    }
}
