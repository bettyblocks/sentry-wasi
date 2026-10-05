# sentry-wasi

Wrapper library around sentry to make it usable in wasm components.

Also reexports sentry library for ease of use.

Implements a custom Sentry transport for either WASI p2 (via `wstd`)
or WASI p3 (via `wasi-fetch`), selected with Cargo features.

```toml
[dependencies]
# WASI p2
sentry-wasi = { version = "0.2", features = ["wasip2"] }

# WASI p3
sentry-wasi = { version = "0.2", features = ["wasip3"] }
```

```rust
use sentry_wasi::sentry;

// Setup sentry connection
let _guard = sentry::init(sentry::ClientOptions {
    dsn: "http://testkey@127.0.0.1:9992/1".parse().ok(),
    release: Some("wasi-demo@0.1.0".into()),
    environment: Some("development".into()),
    debug: true,
    // here use the custom transport
    transport: Some(sentry_wasi::create_transport()),
    ..Default::default()
});

// use sentry library normally
sentry::configure_scope(|scope| {
    scope.set_tag("component", "wasi-demo");
});
```

## Awaiting delivery (WASI p3)

`create_transport()` sends from a detached task: if the component call returns
first, the event is lost. `create_queued_transport()` queues envelopes and
`sentry_wasi::flush().await` sends them and reports the HTTP statuses:

```rust
let _guard = sentry::init(sentry::ClientOptions {
    transport: Some(sentry_wasi::create_queued_transport()),
    ..Default::default()
});
sentry::capture_message("boom", sentry::Level::Error);
let report = sentry_wasi::flush().await; // report.statuses, report.errors
```

Based on: [wasi-sentry-demo](https://github.com/Aditya1404Sal/wasi-sentry-demo)
