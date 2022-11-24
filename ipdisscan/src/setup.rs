use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::Report;
use std::path::Path;
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

const DEFAULT_LOG_LVL: &str = "info";

pub fn eyre_setup() -> Result<(), Report> {
    color_eyre::install()?;
    Ok(())
}

pub fn log_setup(log_file: &Path) -> Result<(), Report> {
    install_file_tracing(log_file)
}

fn install_file_tracing(log_file: &Path) -> Result<(), Report> {
    let filter_layer = get_envfilter(DEFAULT_LOG_LVL);
    let file_appender = tracing_appender::rolling::never(
        log_file
            .parent()
            .wrap_err_with(|| format!("{:?} path has no parent", log_file))?,
        log_file
            .file_stem()
            .wrap_err_with(|| format!("{:?} path has no file name", log_file))?,
    );
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_writer(non_blocking_appender);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(ErrorLayer::default())
        .with(fmt_layer)
        .init();
    Ok(())
}

/// Logging levels configuration as per
/// https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives
fn get_envfilter(default: &str) -> EnvFilter {
    EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default))
        .unwrap()
}
