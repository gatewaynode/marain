use crate::EnvPaths;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, time::OffsetTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Initialize the logging system with file output to data/logs directory
pub fn init_logging(
    env_paths: &EnvPaths,
) -> Result<tracing_appender::non_blocking::WorkerGuard, Box<dyn std::error::Error>> {
    // Get the logs directory path from environment paths
    let logs_dir = env_paths.data_path.join("logs");

    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&logs_dir)?;

    // Create a rolling file appender that creates a new log file daily
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("marain-cms")
        .filename_suffix("log")
        .build(&logs_dir)?;

    // Create a non-blocking writer
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Set up time formatting with local time
    let timer = OffsetTime::local_rfc_3339().unwrap_or_else(|_| {
        // Fallback to UTC if local time fails (can happen in some environments)
        OffsetTime::new(
            time::UtcOffset::UTC,
            time::format_description::well_known::Rfc3339,
        )
    });

    // Create the subscriber with both file and console output
    let subscriber = tracing_subscriber::registry()
        // File layer with full details
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_timer(timer.clone())
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true),
        )
        // Console layer for development
        .with(
            fmt::layer()
                .with_timer(timer)
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false),
        )
        // Environment filter
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    // Initialize the subscriber
    subscriber.init();

    tracing::info!("Logging system initialized");
    tracing::info!(
        "Log files are being written to: {:?}",
        env_paths.data_path.join("logs")
    );

    Ok(guard)
}

/// Log application shutdown
pub fn log_shutdown() {
    tracing::info!("Application shutting down");
    tracing::info!("=== Marain CMS shutdown complete ===");
}
