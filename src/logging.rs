//! # Logging System
//!
//! Structured logging with tracing for SilverBitcoin node.

use crate::config::LoggingConfig;
use std::path::Path;
use thiserror::Error;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};
use tracing_appender::{non_blocking, rolling};

/// Logging error types
#[derive(Error, Debug)]
pub enum LoggingError {
    /// Failed to create log directory
    #[error("Failed to create log directory: {0}")]
    DirectoryError(#[from] std::io::Error),

    /// Invalid log level
    #[error("Invalid log level: {0}")]
    InvalidLevel(String),

    /// Failed to initialize logging
    #[error("Failed to initialize logging: {0}")]
    InitError(String),
}

/// Result type for logging operations
pub type Result<T> = std::result::Result<T, LoggingError>;

/// Initialize logging system
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    // Parse log level
    let level = parse_log_level(&config.level)?;

    // Create log directory if it doesn't exist
    if let Some(parent) = config.log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create file appender with rotation
    let file_name = config.log_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("node.log");
    
    let directory = config.log_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let file_appender = rolling::RollingFileAppender::builder()
        .rotation(rolling::Rotation::DAILY)
        .filename_prefix(file_name)
        .max_log_files(config.max_log_files)
        .build(directory)
        .map_err(|e| LoggingError::InitError(format!("Failed to create file appender: {}", e)))?;

    let (non_blocking_appender, _guard) = non_blocking(file_appender);

    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new(format!("silver={},silver_node={}", config.level, config.level))
        });

    // Build subscriber based on format preference
    if config.json_format {
        // JSON structured logging
        let file_layer = fmt::layer()
            .json()
            .with_writer(non_blocking_appender)
            .with_span_events(FmtSpan::CLOSE)
            .with_current_span(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_target(true)
            .with_file(true)
            .with_line_number(true);

        let stdout_layer = fmt::layer()
            .json()
            .with_span_events(FmtSpan::CLOSE)
            .with_current_span(true)
            .with_target(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    } else {
        // Human-readable logging
        let file_layer = fmt::layer()
            .with_writer(non_blocking_appender)
            .with_span_events(FmtSpan::CLOSE)
            .with_ansi(false)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true);

        let stdout_layer = fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_ansi(true)
            .with_target(true)
            .compact();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    }

    // Log initialization message
    tracing::info!(
        log_level = %config.level,
        log_path = ?config.log_path,
        json_format = config.json_format,
        max_log_files = config.max_log_files,
        "Logging system initialized"
    );

    Ok(())
}

/// Parse log level string
fn parse_log_level(level: &str) -> Result<Level> {
    match level.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(LoggingError::InvalidLevel(level.to_string())),
    }
}

/// Log critical operation
#[macro_export]
macro_rules! log_critical {
    ($($arg:tt)*) => {
        tracing::error!(critical = true, $($arg)*)
    };
}

/// Log state transition
#[macro_export]
macro_rules! log_state_transition {
    ($from:expr, $to:expr) => {
        tracing::info!(
            from = ?$from,
            to = ?$to,
            "State transition"
        )
    };
}

/// Log performance metric
#[macro_export]
macro_rules! log_metric {
    ($name:expr, $value:expr) => {
        tracing::debug!(
            metric = $name,
            value = $value,
            "Performance metric"
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_level() {
        assert!(parse_log_level("trace").is_ok());
        assert!(parse_log_level("debug").is_ok());
        assert!(parse_log_level("info").is_ok());
        assert!(parse_log_level("warn").is_ok());
        assert!(parse_log_level("error").is_ok());
        assert!(parse_log_level("invalid").is_err());
    }

    #[test]
    fn test_parse_log_level_case_insensitive() {
        assert!(parse_log_level("INFO").is_ok());
        assert!(parse_log_level("Info").is_ok());
        assert!(parse_log_level("iNfO").is_ok());
    }
}
