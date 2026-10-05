//! Wrapper library around sentry to make it usable in wasm components.
//!
//! Also reexports sentry library for ease of use.
//!
//! Implements custom Sentry transport using wstd.
//!
//! ## Awaiting delivery (`wasip3`)
//!
//! [`create_transport`] sends from a detached task, which is lost if the
//! component call ends first. [`create_queued_transport`] queues instead;
//! `.await` [`flush`] before returning to send and get the HTTP status:
//!
//! ```ignore
//! sentry::capture_message("boom", sentry::Level::Error);
//! let report = sentry_wasi::flush().await;
//! ```
//!
//! ```rust
//! use sentry_wasi::sentry;
//!
//! // Setup sentry connection
//! let _guard = sentry::init(sentry::ClientOptions {
//!     dsn: "http://testkey@127.0.0.1:9992/1".parse().ok(),
//!     release: Some("wasi-demo@0.1.0".into()),
//!     environment: Some("development".into()),
//!     debug: true,
//!     // here use the custom transport
//!     transport: Some(sentry_wasi::create_transport()),
//!     ..Default::default()
//! });
//!
//! // use sentry library normally
//! sentry::configure_scope(|scope| {
//!     scope.set_tag("component", "wasi-demo");
//! });
//! ```

pub use sentry;

pub mod transport;

use sentry::{ClientOptions, Transport, TransportFactory};
use std::sync::Arc;
use transport::WasiTransport;

#[cfg(feature = "wasip3")]
pub use transport::{FlushReport, QueuedWasiTransport, flush};

/// transport that can be used in wasm components
pub fn create_transport() -> Arc<dyn TransportFactory> {
    Arc::new(|options: &ClientOptions| -> Arc<dyn Transport> {
        Arc::new(WasiTransport::new(options))
    })
}

/// Like [`create_transport`], but envelopes are queued until [`flush`] is
/// awaited, so delivery can be awaited and its status inspected.
#[cfg(feature = "wasip3")]
pub fn create_queued_transport() -> Arc<dyn TransportFactory> {
    Arc::new(|options: &ClientOptions| -> Arc<dyn Transport> {
        Arc::new(QueuedWasiTransport::new(options))
    })
}
