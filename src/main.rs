use jokolay::{client::am::{ASSETS_FOLDER_NAME, LOG_FILE_NAME}, config::JokoConfig, core::JokoCore, show_msg_box};
use std::{path::PathBuf, str::FromStr};
fn main() {
    loop {
        if core_main() {
            break;
        }
    }
}
fn core_main() -> bool {
    let (mut config, assets_path) = setup();
    let mut app = JokoCore::new(&mut config, assets_path).unwrap();
    log::trace!("Jokolay App initialized.");
    std::thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            println!("Hello world");
        })
    });
    app.run().unwrap();
    true
}


fn setup() -> (JokoConfig, PathBuf) {
    let assets_path = std::env::var("JOKO_ASSETS").unwrap_or_else(|_| {
        std::env::current_dir()
            .map_err(|e| {
                show_msg_box(
                    "jokolay error",
                    &format!(
                        "could not get current directory from env due to error {:?}",
                        &e
                    ),
                    rfd::MessageButtons::Ok,
                    rfd::MessageLevel::Error,
                );
                e
            })
            .unwrap()
            .join(ASSETS_FOLDER_NAME)
            .to_string_lossy()
            .to_string()
    });
    let assets_path = PathBuf::from_str(&assets_path)
        .map_err(|e| {
            show_msg_box(
                "assets_path erro",
                &format!("assets path could not be parsed due to error {:?}", &e),
                rfd::MessageButtons::Ok,
                rfd::MessageLevel::Error,
            );
            e
        })
        .unwrap();
    if !assets_path.exists() {
        std::fs::create_dir_all(&assets_path)
            .map_err(|e| {
                show_msg_box(
                    "couldn't create assets folder",
                    &format!(
                    "failed to create assets directory when it doesn't exist, due to error: {:?}",
                    &e
                ),
                    rfd::MessageButtons::Ok,
                    rfd::MessageLevel::Error,
                );
                e
            })
            .unwrap();
    }

    let config_file_path = assets_path.join("joko_config.json");
    let config_file = std::fs::File::open(&config_file_path)
        .map_err(|_e| {
            {
                let config = JokoConfig::default();
                let cf = std::fs::File::create(&config_file_path)
                    .map_err(|e| {
                        show_msg_box(
                            "could not create config",
                            &format!(
                                "failed to create config file when it didn't exist, error: {:?}",
                                &e
                            ),
                            rfd::MessageButtons::Ok,
                            rfd::MessageLevel::Error,
                        );
                        e
                    })
                    .unwrap();
                let cfw = std::io::BufWriter::new(cf);
                serde_json::to_writer(cfw, &config)
                    .expect("failed to write default config to newly created config file");
            }
            std::fs::File::open(&config_file_path)
                .expect("couldn't open config file even after creating it")
        })
        .unwrap();

    let config: JokoConfig = serde_json::from_reader(std::io::BufReader::new(config_file))
        .map_err(|e|
            show_msg_box(
                "config parse error",
             &format!("failed to parse config file due to error: {:?}. please delete it if necessary. config file path: {:?}", &e, &config_file_path),
          rfd::MessageButtons::Ok,
              rfd::MessageLevel::Error)
            )
        .unwrap();
    let log_file_path = assets_path.join(LOG_FILE_NAME);

    jokolay::log_init(
        log::LevelFilter::from_str(&config.term_log_level).unwrap(),
        log::LevelFilter::from_str(&config.file_log_level).unwrap(),
        log_file_path.clone(),
    )
    .unwrap();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{:#?}", info);
        notify_rust::Notification::new()
            .summary("Jokolay")
            .body(&format!(
                "Jokolay crashed. logfile at '{:?}'.",
                log_file_path
            ))
            .hint(notify_rust::Hint::Category("Jokolay".to_owned()))
            .hint(notify_rust::Hint::Resident(true)) // this is not supported by all implementations
            .timeout(notify_rust::Timeout::Never)
            .urgency(notify_rust::Urgency::Critical)
            .show()
            .unwrap();
    }));
    
    (config, assets_path)
}

