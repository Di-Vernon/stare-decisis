//! Tracing subscriber initialisation.
//!
//! Every myth binary calls `init_logging(env!("CARGO_BIN_NAME"))` at the
//! top of `main`. Output goes to stderr so stdout stays reserved for
//! hook protocol JSON. Safe to call more than once — subsequent calls
//! silently no-op via `try_init`.

use tracing_subscriber::EnvFilter;

pub fn init_logging(binary_name: &str) {
    let filter = EnvFilter::try_from_env("MYTH_LOG")
        .unwrap_or_else(|_| EnvFilter::new("myth=info,warn"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .compact()
        .try_init();

    tracing::info!(binary = binary_name, "myth starting");
}
