use std::time::Instant;
use std::{env, io};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::EnvFilter;

struct Uptime(Instant);

impl Uptime {
    #[inline] #[must_use]
    fn new() -> Self { Self(Instant::now()) }
}

impl FormatTime for Uptime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let elapsed = self.0.elapsed();
        let s = elapsed.as_secs();
        match cfg!(debug_assertions) {
            true => write!(w, "{s:>4}.{us:06}", us = elapsed.subsec_micros()),
            false => write!(w, "{s:>4}.{ms:03}", ms = elapsed.subsec_millis()),
        }
    }
}

pub fn log() {
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| String::from("info"));

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(format!("warn,{}={log_level}", env!("CARGO_PKG_NAME")));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_level(true)
        .with_target(cfg!(debug_assertions))
        .with_line_number(cfg!(debug_assertions))
        .with_timer(Uptime::new())
        .with_writer(io::stderr)
        .compact()
        .init();
}
