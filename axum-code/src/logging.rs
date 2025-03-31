use crate::config::Config;
use crate::error::AppError;
use tracing_subscriber::{
    fmt, prelude::*, registry, EnvFilter,
};

pub fn init_logging(config: &Config) -> Result<(), AppError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));

    let formatting_layer = match config.logging.format.as_str() {
        "json" => fmt::layer().json().boxed(),
        _ => fmt::layer().pretty().boxed(),
    };

    registry()
        .with(env_filter)
        .with(formatting_layer)
        .try_init()
        .map_err(|e| AppError::Internal(format!("Failed to initialize logging: {}", e)))?;

    Ok(())
}
