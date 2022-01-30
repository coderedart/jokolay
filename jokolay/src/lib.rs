use anyhow::Context;
use std::path::Path;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
pub mod config;
pub mod core;

pub fn log_initialize(
    assets_dir: &Path,
    log_level: tracing::level_filters::LevelFilter,
) -> anyhow::Result<WorkerGuard> {
    // let file_appender = tracing_appender::rolling::never(log_directory, log_file_name);
    let file_path = assets_dir.join("jokolay.log");
    let writer = std::io::BufWriter::new(
        std::fs::File::create(&file_path)
            .with_context(|| format!("failed to create logfile at path: {:#?}", &file_path))?,
    );
    let (nb, guard) = tracing_appender::non_blocking(writer);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(nb)
        .with_max_level(log_level)
        .init();

    info!("Application Name: {}", env!("CARGO_PKG_NAME"));
    info!("Application Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Application Authors: {}", env!("CARGO_PKG_AUTHORS"));
    info!(
        "Application Repository Link: {}",
        env!("CARGO_PKG_REPOSITORY")
    );
    info!("Application License: {}", env!("CARGO_PKG_LICENSE"));
    Ok(guard)
}
