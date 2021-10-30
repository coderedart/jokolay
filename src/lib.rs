pub mod client;
pub mod config;
pub mod core;
pub mod server;

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Context;
use egui::CtxRef;
use log::LevelFilter;

use rfd::{MessageButtons, MessageDialog, MessageLevel};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    client::tc::{ASSETS_FOLDER_NAME, LOG_FILE_NAME},
    config::JokoConfig,
    core::JokoCore,
};

pub async fn save_egui_memory(ctx: CtxRef, path: &Path) -> anyhow::Result<()> {
    let mut egui_cache = File::create(path).await?;
    let memory = ctx.memory().clone();
    let string = serde_json::to_string_pretty(&memory)?;
    egui_cache.write_all(string.as_bytes()).await?;
    Ok(())
}

pub async fn save_config(config: &JokoConfig, path: &Path) -> anyhow::Result<()> {
    let mut config_file = File::create(path).await?;
    let config_string = serde_json::to_string_pretty(config)?;
    config_file.write_all(config_string.as_bytes()).await?;
    Ok(())
}

/// initializes global logging backend that is used by log macros
/// Takes in a filter for stdout/stderr, a filter for logfile and finally the path to logfile
pub fn log_init(
    term_filter: LevelFilter,
    file_filter: LevelFilter,
    file_path: PathBuf,
) -> anyhow::Result<()> {
    use simplelog::*;
    use std::fs::File;
    let config = ConfigBuilder::new()
        .set_location_level(LevelFilter::Error)
        .build();

    CombinedLogger::init(vec![
        TermLogger::new(term_filter, config, TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(file_filter, Config::default(), File::create(file_path)?),
    ])?;
    Ok(())
}

#[macro_export]
macro_rules! gl_error {
    ($gl:expr) => {
        let e = $gl.get_error();
        if e != glow::NO_ERROR {
            log::error!("glerror {} at {} {} {}", e, file!(), line!(), column!());
        }
    };
}

pub fn show_msg_box(title: &str, msg: &str, buttons: MessageButtons, lvl: MessageLevel) -> bool {
    MessageDialog::new()
        .set_level(lvl)
        .set_title(title)
        .set_description(msg)
        .set_buttons(buttons)
        .show()
}

pub fn real_main() -> bool {
    let (mut config, assets_path) = setup()
        .map_err(|e| {
            rfd::MessageDialog::new()
                .set_title("jokolay could not start")
                .set_description(&format!("{}", &e))
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        })
        .unwrap();

    let mut app = JokoCore::new(&mut config, assets_path).unwrap();
    log::trace!("Jokolay App initialized.");

    app.run().expect("app failed to run");
    false
}

pub fn setup() -> anyhow::Result<(JokoConfig, PathBuf)> {
    let assets_path = match std::env::var("JOKO_ASSETS") {
        Ok(ap) => ap,
        Err(_) => std::env::current_dir()
            .context("failed to get current_dir()")?
            .join(ASSETS_FOLDER_NAME)
            .to_string_lossy()
            .to_string(),
    };
    let assets_path = PathBuf::from_str(&assets_path).context("assets path could not be parsed")?;
    if !assets_path.exists() {
        std::fs::create_dir_all(&assets_path).context("failed to create assets directory")?;
    }

    let config_file_path = assets_path.join("joko_config.json");
    if !config_file_path.exists() {
        let config = JokoConfig::default();
        let cf = std::fs::File::create(&config_file_path)
            .context("could not create joko_config.json")?;
        let cfw = std::io::BufWriter::new(cf);
        serde_json::to_writer(cfw, &config)
            .context("failed to write default config to newly created joko_config.json")?;
    }
    let config_file = std::fs::File::open(&config_file_path).unwrap();

    let config: JokoConfig = serde_json::from_reader(std::io::BufReader::new(config_file))
        .context("error when trying to parse config file. please delete joko_config.json")?;

    let log_file_path = assets_path.join(LOG_FILE_NAME);
    log_init(
        log::LevelFilter::from_str(&config.term_log_level)
            .context("failed to deserialize term log level from config")?,
        log::LevelFilter::from_str(&config.file_log_level)
            .context("failed to deserialize file log level from config")?,
        log_file_path.clone(),
    )
    .context("failed to log initialize")?;
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{:#?}", info);
        let _ = notify_rust::Notification::new()
            .summary("Jokolay Crash Error")
            .body(&format!("crashed due to: {:?}.", info))
            .hint(notify_rust::Hint::Category("Jokolay".to_owned()))
            .hint(notify_rust::Hint::Resident(true)) // this is not supported by all implementations
            .timeout(notify_rust::Timeout::Never)
            .urgency(notify_rust::Urgency::Critical)
            .show();
        let _ = notify_rust::Notification::new()
            .summary("Jokolay")
            .body(&format!(
                "Jokolay crashed. logfile at '{:?}'.",
                log_file_path
            ))
            .hint(notify_rust::Hint::Category("Jokolay".to_owned()))
            .hint(notify_rust::Hint::Resident(true)) // this is not supported by all implementations
            .timeout(notify_rust::Timeout::Never)
            .urgency(notify_rust::Urgency::Critical)
            .show();
    }));

    Ok((config, assets_path))
}
