use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;
use crate::errors::ConfigError;

/// Sets up the logger
///
/// # Arguments
///
/// * 'log_path' - path where to save logs
/// * 'log_to_stdout' - whether to log to stdout or not
pub fn setup_logger(log_path: &str, log_to_stdout: bool) -> Result<(), ConfigError> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{d(%Y-%m-%dT%H:%M:%S):0<19}{d(%:z)} {l} {M}] - {m}{n}")))
        .build();

    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{d(%Y-%m-%dT%H:%M:%S):0<19}{d(%:z)} {l} {M}] - {m}{n}")))
        .build(log_path)?;


    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("file", Box::new(file)))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder()
            .appenders(["file","stdout"]).build(LevelFilter::Info)

        )?;

    let _ = log4rs::init_config(config)?;

    Ok(())
}
