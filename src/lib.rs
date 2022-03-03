use anyhow::Context;
use std::path::Path;
use tracing::warn;
use tracing_appender::non_blocking::WorkerGuard;
pub mod config;
pub mod core;

pub fn log_initialize(
    log_file_path: &Path,
    log_level: tracing::level_filters::LevelFilter,
) -> anyhow::Result<WorkerGuard> {
    // let file_appender = tracing_appender::rolling::never(log_directory, log_file_name);
    let writer = std::io::BufWriter::new(
        std::fs::File::create(&log_file_path)
            .with_context(|| format!("failed to create logfile at path: {:#?}", &log_file_path))?,
    );
    let (nb, guard) = tracing_appender::non_blocking(writer);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(nb)
        .with_max_level(log_level)
        .init();

    warn!("Application Name: {}", env!("CARGO_PKG_NAME"));
    warn!("Application Version: {}", env!("CARGO_PKG_VERSION"));
    warn!("Application Authors: {}", env!("CARGO_PKG_AUTHORS"));
    warn!(
        "Application Repository Link: {}",
        env!("CARGO_PKG_REPOSITORY")
    );
    warn!("Application License: {}", env!("CARGO_PKG_LICENSE"));
    Ok(guard)
}

pub fn get_config_data_cache_markers_dirs() -> anyhow::Result<[std::path::PathBuf; 7]> {
    let current_dir =
        std::env::current_dir().context("failed to get current directory from env")?;
    let config_dir_path = current_dir.join("config");
    let data_dir_path = current_dir.join("data");
    let cache_dir_path = current_dir.join("cache");
    let markers_dir_path = current_dir.join("markers_dir_path");
    let logs_dir_path = current_dir.join("logs");
    let themes_dir_path = data_dir_path.join("themes");
    let fonts_dir_path = data_dir_path.join("fonts");
    let result = [
        config_dir_path,
        data_dir_path,
        cache_dir_path,
        markers_dir_path,
        logs_dir_path,
        themes_dir_path,
        fonts_dir_path,
    ];
    for p in &result {
        std::fs::create_dir_all(p).context("failed to setup directories")?;
    }
    Ok(result)
}
