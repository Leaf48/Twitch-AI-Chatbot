use log::LevelFilter;

pub struct LoggerSetup {}

impl LoggerSetup {
    pub fn new() -> Self {
        let mut builder = env_logger::builder();
        // hide logs except this module
        builder.filter(None, LevelFilter::Error);
        builder.filter_module(env!("CARGO_CRATE_NAME"), get_log_level());
        builder.init();

        Self {}
    }
}

/// Get RUST_LOG as LevelFilter
fn get_log_level() -> LevelFilter {
    let level = std::env::var("RUST_LOG")
        .expect("RUST_LOG is undefined")
        .to_lowercase();

    match level.as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    }
}
