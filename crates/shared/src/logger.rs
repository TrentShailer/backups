use std::{fs::create_dir_all, io};

use thiserror::Error;
use tracing::{Level, subscriber::set_global_default};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{layer::SubscriberExt, registry};

/// Create and set the global loggers.
pub fn init_logger() -> Result<Vec<WorkerGuard>, LoggerError> {
    create_dir_all("./logs").map_err(LoggerError::CreateDirectory)?;

    let filter = tracing_subscriber::filter::Targets::new().with_default(Level::INFO);

    // File layer
    let (file_guard, file_layer) = {
        let appender = RollingFileAppender::builder()
            .filename_suffix("log")
            .rotation(Rotation::DAILY)
            .max_log_files(90)
            .build("./logs")?;

        let (writer, guard) = tracing_appender::non_blocking(appender);

        let layer = tracing_subscriber::fmt::layer()
            .with_writer(writer)
            .with_ansi(false)
            .with_target(false);

        (guard, layer)
    };

    // Std layer
    let (std_guard, std_layer) = {
        let (writer, guard) = tracing_appender::non_blocking(io::stdout());

        let layer = tracing_subscriber::fmt::layer()
            .with_writer(writer)
            .with_ansi(true)
            .with_target(false);

        (guard, layer)
    };

    // Create registry
    let registry = registry().with(file_layer).with(std_layer).with(filter);

    // Set global subscriber
    set_global_default(registry).unwrap();

    Ok(vec![file_guard, std_guard])
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum LoggerError {
    #[error("Failed to create rolling appender:\n{0}")]
    CreateRollingAppender(#[from] tracing_appender::rolling::InitError),

    #[error("Failed to create log directory:\n{0}")]
    CreateDirectory(#[source] io::Error),
}
