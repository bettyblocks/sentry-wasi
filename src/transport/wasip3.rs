use sentry::{ClientOptions, Envelope, Transport};
use wasi_fetch::Client;

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
