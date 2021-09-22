use jokolay::{core::JokoConfig, JokolayApp};
use std::{io::Write, str::FromStr};
fn main() -> anyhow::Result<()> {
    let config_file = std::fs::File::open("./joko_config.json");
    let mut config = JokoConfig::default();
    let mut log = std::fs::File::create("./jokolay.log").unwrap();

    match config_file {
        Err(e) => {
            write!(&mut log, "could not open joko_config.json file in {:?} due to error {:?}. trying to create it.", std::env::current_dir().unwrap(), &e).unwrap();
            let config_file = std::fs::File::create("./joko_config.json");
            match config_file {
                Ok(f) => {
                    let writer = std::io::BufWriter::new(f);
                    serde_json::to_writer(writer, &config).unwrap();
                }
                Err(_) => {
                    log.write_fmt(format_args!("failed to create config_file. exiting"))
                        .unwrap();
                    return Ok(());
                }
            }
        }
        Ok(f) => {
            config = serde_json::from_reader(std::io::BufReader::new(f))
                .map_err(|e| {
                    log.write_fmt(format_args!(
                        "failed to parse joko_config.json due to {:?}",
                        &e
                    ))
                    .unwrap();
                    e
                })
                .unwrap();
        }
    }

    jokolay::log_init(
        log::LevelFilter::from_str(&config.term_log_level).unwrap(),
        log::LevelFilter::from_str(&config.file_log_level).unwrap(),
        config.log_file_path.clone(),
    )?;
    let app = JokolayApp::new(config);
    log::trace!("app initialized.");
    app.run()?;
    Ok(())
}
