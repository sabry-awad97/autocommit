use log::LevelFilter;

pub fn init_logger(level: &str) {
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| level.to_string());

    let level_filter = match log_level.as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new()
        .filter_level(level_filter)
        .format_timestamp(None)
        .init();
}
