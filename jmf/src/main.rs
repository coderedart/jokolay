use tracing::info;

fn main() -> color_eyre::Result<()> {
    {
        let _guard = {
            use tracing_error::ErrorLayer;
            use tracing_subscriber::prelude::*;
            use tracing_subscriber::{fmt, EnvFilter};
            let file_path = std::path::Path::new(".").join("jmf.log");
            let writer =
                std::io::BufWriter::new(std::fs::File::create(&file_path).unwrap_or_else(|e| {
                    panic!(
                        "failed to create logfile at path: {:#?} due to error: {:#?}",
                        &file_path, &e
                    )
                }));
            let (nb, guard) = tracing_appender::non_blocking(writer);

            let fmt_layer = fmt::layer()
                .with_target(true)
                .with_ansi(false)
                .with_writer(nb);
            let filter_layer = EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("info"))
                .unwrap();

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer)
                .with( ErrorLayer::default())
                .init();
            color_eyre::install()?;
            info!("Application Name: {}", env!("CARGO_PKG_NAME"));
            info!("Application Version: {}", env!("CARGO_PKG_VERSION"));
            info!("Application Authors: {}", env!("CARGO_PKG_AUTHORS"));
            info!(
                "Application Repository Link: {}",
                env!("CARGO_PKG_REPOSITORY")
            );
            info!("Application License: {}", env!("CARGO_PKG_LICENSE"));

            info!("git version details: {}", git_version::git_version!());

            info!("created app and initialized logging");
            guard
        };

        let (pack, warnings, errors) =
            jmf::xmlpack::load::xml_to_json_pack(std::path::Path::new("./assets/packs/tw"));
        std::thread::sleep(std::time::Duration::from_secs(5));

        tracing::warn!("{:#?}", &warnings);
        tracing::error!("{:#?}", &errors);
        std::fs::create_dir_all("./assets/packs/json_tw").unwrap();
        pack.pack.save_to_folder(std::path::Path::new("./assets/packs/json_tw")).unwrap();

        // serde_json::to_writer_pretty(
        //     std::io::BufWriter::new(std::fs::File::create("./assets/packs/pack.json").unwrap()),
        //     &pack.pack,
        // )
        // .unwrap();
        // let pack_file = std::io::BufReader::new( std::fs::File::open("./assets/packs/pack.json").unwrap());
        // let pack: Pack = serde_json::from_reader(pack_file).unwrap();
        // std::thread::sleep(std::time::Duration::from_secs(30));
        Ok(())
    }
}
