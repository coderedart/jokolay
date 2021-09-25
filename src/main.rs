use jokolay::{
    core::{
        fm::{ASSETS_FOLDER_NAME, LOG_FILE_NAME},
        JokoConfig,
    },
    show_msg_box, JokolayApp,
};
use std::{path::PathBuf, str::FromStr};
fn main() -> anyhow::Result<()> {
    let assets_path = std::env::var("JOKO_ASSETS").unwrap_or_else(|_| {
        std::env::current_dir()
            .unwrap_or_else(|e| {
                show_msg_box(
                    "jokolay error",
                    &format!(
                        "could not get current directory from env due to error {:?}",
                        &e
                    ),
                    rfd::MessageButtons::Ok,
                    rfd::MessageLevel::Error,
                );

                panic!()
            })
            .join(ASSETS_FOLDER_NAME)
            .to_string_lossy()
            .to_string()
    });
    let assets_path = PathBuf::from_str(&assets_path).unwrap_or_else(|e| {
        show_msg_box(
            "assets_path erro",
            &format!("assets path could not be parsed due to error {:?}", &e),
            rfd::MessageButtons::Ok,
            rfd::MessageLevel::Error,
        );
        panic!()
    });
    if !assets_path.exists() {
        std::fs::create_dir_all(&assets_path).unwrap_or_else(|e| {
            show_msg_box(
                "couldn't create assets folder",
                &format!(
                    "failed to create assets directory when it doesn't exist, due to error: {:?}",
                    &e
                ),
                rfd::MessageButtons::Ok,
                rfd::MessageLevel::Error,
            );
            panic!()
        });
    }

    let config_file_path = assets_path.join("joko_config.json");
    let config_file = std::fs::File::open(&config_file_path).unwrap_or_else(|_| {
        {
            let config = JokoConfig::default();
            let cf = std::fs::File::create(&config_file_path).unwrap_or_else(|e| {
                show_msg_box(
                    "could not create config",
                    &format!(
                        "failed to create config file when it didn't exist, error: {:?}",
                        &e
                    ),
                    rfd::MessageButtons::Ok,
                    rfd::MessageLevel::Error,
                );
                panic!()
            });
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
