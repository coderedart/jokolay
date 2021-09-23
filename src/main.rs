use jokolay::{
    core::{
        fm::{ASSETS_FOLDER_NAME, LOG_FILE_NAME},
        JokoConfig,
    },
    JokolayApp,
};
use std::{path::PathBuf, str::FromStr};
fn main() -> anyhow::Result<()> {
    let assets_path = std::env::var("JOKO_ASSETS").unwrap_or_else(|_| {
        std::env::current_dir()
            .expect("couldn't get current dir")
            .join(ASSETS_FOLDER_NAME)
            .to_string_lossy()
            .to_string()
    });
    let assets_path = PathBuf::from_str(&assets_path).expect("assets path could not be parsed");
    if !assets_path.exists() {
        std::fs::create_dir_all(&assets_path)
            .expect("failed to create assets directory when it doesn't exist");
    }
    let config_file_path = assets_path.join("joko_config.json");
    let config_file = std::fs::File::open(&config_file_path).unwrap_or_else(|_| {
        {
            let config = JokoConfig::default();
            let cf = std::fs::File::create(&config_file_path)
                .expect("failed to create config file when it didn't exist");
            let cfw = std::io::BufWriter::new(cf);
            serde_json::to_writer(cfw, &config)
                .expect("failed to write default config to newly created config file");
        }
        std::fs::File::open(&config_file_path)
            .expect("couldn't open config file even after creating it")
    });

    let config: JokoConfig = serde_json::from_reader(std::io::BufReader::new(config_file))
        .expect("failed to parse config file. please delete it if necessary");

    jokolay::log_init(
        log::LevelFilter::from_str(&config.term_log_level).unwrap(),
        log::LevelFilter::from_str(&config.file_log_level).unwrap(),
        assets_path.join(LOG_FILE_NAME),
    )?;
    let app = JokolayApp::new(config, assets_path);
    log::trace!("app initialized.");
    app.run()?;
    Ok(())
}
