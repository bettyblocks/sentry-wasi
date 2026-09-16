use sentry::{ClientOptions, Envelope, Transport};
use wstd::http::{Body, Client, Method, Request};
use wstd::runtime::spawn;

/// Sentry transport that can be used in wasm components
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

        spawn(async move {
            let request = Request::builder()
                .method(Method::POST)
                .uri(url.as_str())
                .header("X-Sentry-Auth", &auth)
                .body(Body::from(body))
                .expect("unable to create request");

            match Client::new().send(request).await {
                Ok(_) => eprintln!("sentry: envelope sent successfully"),
                Err(err) => eprintln!("sentry: failed to send envelope: {err}"),
            }
        })
        .detach();
    }
}
