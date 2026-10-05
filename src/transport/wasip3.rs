use sentry::{ClientOptions, Envelope, Transport};
use std::sync::Mutex;
use std::time::Duration;
use wasi_fetch::Client;

const SEND_TIMEOUT: Duration = Duration::from_secs(10);

/// Sentry transport that can be used in wasm components, using WASI p3's
/// native async HTTP support via the `wasi-fetch` crate.
pub struct WasiTransport {
    url: String,
    auth: String,
}

impl WasiTransport {
    pub fn new(options: &ClientOptions) -> Self {
        let dsn = options.dsn.as_ref().expect("dns needs to be set");
        let auth = dsn.to_auth(Some(&options.user_agent)).to_string();
        let url = dsn.envelope_api_url().to_string();
        Self { url, auth }
    }
}

impl Transport for WasiTransport {
    fn send_envelope(&self, envelope: Envelope) {
        let mut body = Vec::new();
        envelope
            .to_writer(&mut body)
            .expect("failing to write to Vec");

        let url = self.url.clone();
        let auth = self.auth.clone();

        wasip3::spawn(async move {
            let result = Client::new()
                .post(url.as_str())
                .header("X-Sentry-Auth", auth.as_str())
                .body(body)
                .send()
                .await;

            match result {
                Ok(_) => eprintln!("sentry: envelope sent successfully"),
                Err(err) => eprintln!("sentry: failed to send envelope: {err}"),
            }
        });
    }
}

struct Queued {
    url: String,
    auth: String,
    body: Vec<u8>,
}

/// Envelopes captured by a [`QueuedWasiTransport`] and not yet POSTed. A static
/// is fine: a component instance lives for one call.
static QUEUE: Mutex<Vec<Queued>> = Mutex::new(Vec::new());

/// Sentry transport that *queues* envelopes instead of spawning the POST, so
/// the caller can await delivery with [`flush`].
///
/// [`WasiTransport`] spawns a detached task that only runs while the component
/// call is alive, and gives the caller nothing to await — an event captured
/// right before the call returns is lost. The sentry crate's own transports
/// avoid that with a worker thread joined on the init guard's drop, which wasm
/// has no equivalent of (the `Transport` trait is sync and WASI HTTP is async).
/// Nothing is sent until [`flush`] is awaited.
pub struct QueuedWasiTransport {
    url: String,
    auth: String,
}

impl QueuedWasiTransport {
    pub fn new(options: &ClientOptions) -> Self {
        let dsn = options.dsn.as_ref().expect("dns needs to be set");
        let auth = dsn.to_auth(Some(&options.user_agent)).to_string();
        let url = dsn.envelope_api_url().to_string();
        Self { url, auth }
    }
}

impl Transport for QueuedWasiTransport {
    fn send_envelope(&self, envelope: Envelope) {
        let mut body = Vec::new();
        if envelope.to_writer(&mut body).is_err() {
            return;
        }
        if let Ok(mut queue) = QUEUE.lock() {
            queue.push(Queued {
                url: self.url.clone(),
                auth: self.auth.clone(),
                body,
            });
        }
    }
}

/// Outcome of [`flush`]: one entry per queued envelope.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FlushReport {
    /// HTTP status the Sentry server answered with.
    pub statuses: Vec<u16>,
    /// Transport errors (DNS, connect, timeout, ...).
    pub errors: Vec<String>,
}

impl FlushReport {
    /// Every envelope was accepted (2xx) and none failed. Vacuously true when
    /// nothing was queued.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty() && self.statuses.iter().all(|s| (200..300).contains(s))
    }
}

/// POST every envelope queued by a [`QueuedWasiTransport`] and await the result.
/// Each send is bounded by a 10s timeout.
pub async fn flush() -> FlushReport {
    // Drain first: the lock must not be held across an await.
    let queued = QUEUE
        .lock()
        .map(|mut queue| std::mem::take(&mut *queue))
        .unwrap_or_default();
    let mut report = FlushReport::default();
    for item in queued {
        let result = Client::new()
            .post(item.url.as_str())
            .header("X-Sentry-Auth", item.auth.as_str())
            .body(item.body)
            .timeout(SEND_TIMEOUT)
            .send()
            .await;
        match result {
            Ok(response) => report.statuses.push(response.status().as_u16()),
            Err(err) => report.errors.push(err.to_string()),
        }
    }
    report
}
