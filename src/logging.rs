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
pub fn setup_logger(log_path: &str) -> Result<(), ConfigError> {

    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{d(%Y-%m-%dT%H:%M:%S%.f):0<29}{d(%:z)} {l} {M}] - {m}{n}")))
        .build(log_path)?;


    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(Root::builder()
            .appenders(["file"]).build(LevelFilter::Info)

        )?;

    let _ = log4rs::init_config(config)?;

    Ok(())
}
