pub mod client;
pub mod config;
pub mod core;
pub mod server;

use std::path::{Path, PathBuf};

use egui::CtxRef;
use log::LevelFilter;

use rfd::{MessageButtons, MessageDialog, MessageLevel};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::config::JokoConfig;

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
    config_file.write_all(config_string.as_bytes());
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
