#[cfg(all(feature = "wasip2", feature = "wasip3"))]
compile_error!("features `wasip2` and `wasip3` are mutually exclusive; enable only one");

#[cfg(not(any(feature = "wasip2", feature = "wasip3")))]
compile_error!("either feature `wasip2` or `wasip3` must be enabled");

#[cfg(feature = "wasip2")]
mod wasip2;
#[cfg(feature = "wasip2")]
pub use wasip2::WasiTransport;

#[cfg(feature = "wasip3")]
mod wasip3;
#[cfg(feature = "wasip3")]
pub use wasip3::{FlushReport, QueuedWasiTransport, WasiTransport, flush};
