use std::path::Path;
use std::sync::OnceLock;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::initialization::ConfigError;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// Sets up the logger
///
/// # Arguments
///
/// * 'log_path' - path where to save logs
/// * 'log_level' - log level
/// * 'log_to_stdout' - whether to log to stdout or not
pub fn setup_logger(log_path: &str, log_level: LevelFilter, log_to_stdout: bool) -> Result<(), ConfigError> {
    let path = Path::new(log_path);
    let file_name = path
        .file_name()
        .ok_or(ConfigError::InvalidLogPathError)?
        .to_string_lossy()
        .to_string();
    let dir = path.parent().unwrap_or_else(|| Path::new("."));

    let (file_writer, guard) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(dir, file_name),
    );
    let _ = LOG_GUARD.set(guard);

    let level = log_level;
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level.to_string()))
        .add_directive("rustls::msgs::handshake=off".parse().unwrap());

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(file_writer)
        .with_timer(fmt::time::ChronoLocal::new("%Y-%m-%dT%H:%M:%S%:z".to_string()))
        .compact();

    let stdout_layer = log_to_stdout.then(|| {
        fmt::layer()
            .with_target(true)
            .with_writer(std::io::stdout)
            .with_timer(fmt::time::ChronoLocal::new("%Y-%m-%dT%H:%M:%S%:z".to_string()))
            .compact()
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .try_init()?;

    Ok(())
}
