# sentry-wasi

Wrapper library around sentry to make it usable in wasm components.

Also reexports sentry library for ease of use.

Implements custom Sentry transport using wstd.

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

Based on: [wasi-sentry-demo](https://github.com/Aditya1404Sal/wasi-sentry-demo)
