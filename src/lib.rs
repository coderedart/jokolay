use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use std::path::Path;
use tracing::warn;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub mod config;
pub mod core;
#[allow(unused_macros)]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::YOUR_STATIC_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::YOUR_STATIC_LOADER, $message_id, $($args), *)
    }};
}

pub fn log_initialize(log_file_path: &Path, log_level: String) -> Result<WorkerGuard> {
    // let file_appender = tracing_appender::rolling::never(log_directory, log_file_name);
    let writer =
        std::io::BufWriter::new(std::fs::File::create(&log_file_path).wrap_err_with(|| {
            format!("failed to create logfile at path: {:#?}", &log_file_path)
        })?);
    let (nb, guard) = tracing_appender::non_blocking(writer);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(nb);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .wrap_err("failed to parse log level :(")?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
    color_eyre::install()?;
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

pub fn get_config_data_cache_markers_dirs() -> Result<[std::path::PathBuf; 6]> {
    let current_dir =
        std::env::current_dir().wrap_err("failed to get current directory from env")?;
    let config_dir_path = current_dir.join("config");
    let data_dir_path = current_dir.join("data");
    let cache_dir_path = current_dir.join("cache");
    let logs_dir_path = current_dir.join("logs");
    let themes_dir_path = data_dir_path.join("themes");
    let fonts_dir_path = data_dir_path.join("fonts");
    let result = [
        config_dir_path,
        data_dir_path,
        cache_dir_path,
        logs_dir_path,
        themes_dir_path,
        fonts_dir_path,
    ];
    for p in &result {
        std::fs::create_dir_all(p).wrap_err("failed to setup directories")?;
    }
    Ok(result)
}
