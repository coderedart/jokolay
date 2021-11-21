use crate::{
    client::JokoClient,
    core::{input::FrameEvents, CoreFrameCommands, JokoCore},
};
use std::{
    path::PathBuf,
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
};

use anyhow::Context;

use log::{trace, LevelFilter};

use crate::{
    client::am::{ASSETS_FOLDER_NAME, LOG_FILE_NAME},
    config::JokoConfig,
};

pub struct JokoApp {
    pub core: Option<JokoCore>,
    pub client: Option<JokoClient>,
    pub soft_restart: Arc<AtomicBool>,
}

impl Default for JokoApp {
    fn default() -> Self {
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

        let (events_sender, events_receiver) = flume::bounded::<FrameEvents>(1000);
        let (commands_sender, commands_receiver) = flume::bounded::<CoreFrameCommands>(1000);

        trace!("Jokolay Setup done");

        let core = JokoCore::new(
            &mut config,
            assets_path.clone(),
            commands_receiver,
            events_sender,
        )
        .unwrap();

        log::trace!("Jokolay Core initialized.");
        let soft_restart = Arc::new(AtomicBool::new(false));
        // create client
        let client = JokoClient::new(
            events_receiver,
            commands_sender,
            soft_restart.clone(),
            assets_path,
        )
        .expect("failed to create JokoClient");
        trace!("jokoclient initialized");

        Self {
            core: Some(core),
            client: Some(client),
            soft_restart,
        }
    }
}
impl JokoApp {
    pub fn run(mut self) {
        loop {
            let client = self.client.take().unwrap();
            let core = self.core.take().unwrap();
            let client_thread = std::thread::spawn(move || {
                client.run();
            });
            core.run();
            client_thread.join().unwrap();
            if self.soft_restart.load(std::sync::atomic::Ordering::Relaxed) {
                log::info!("soft restarting app");
                self = JokoApp::default();
                continue;
            } else {
                break;
            }
        }
    }
}

/// gets the assets folder, gets the configuration file, initializes the logging and sets up the panic hook to show notifications upon crashes
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
            .timeout(notify_rust::Timeout::Never)
            .show();
        let _ = notify_rust::Notification::new()
            .summary("Jokolay")
            .body(&format!(
                "Jokolay crashed. logfile at '{:?}'.",
                log_file_path
            ))
            .timeout(notify_rust::Timeout::Never)
            .show();
    }));

    Ok((config, assets_path))
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

    let _ = CombinedLogger::init(vec![
        TermLogger::new(term_filter, config, TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(file_filter, Config::default(), File::create(file_path)?),
    ]);
    Ok(())
}
