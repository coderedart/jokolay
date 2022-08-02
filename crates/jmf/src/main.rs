use tracing::info;
use tracing_error::ErrorLayer;

fn main() -> color_eyre::Result<()> {
    {
        let _guard = {
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
                .with(ErrorLayer::default())
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

            info!("git version details: {}", jmf::build::SHORT_COMMIT);

            info!("created app and initialized logging");
            guard
        };

        dbg!("before pack deserializing");
        let z = std::fs::read("./assets/tw.zip").expect("failed to read tw zip file");
        let (pack, failures) = jmf::manager::pack::xml::get_pack_from_taco(&z)
            .expect("failed to get pack from zip file");

        tracing::warn!("{:#?}", &failures.warnings);
        tracing::error!("{:#?}", &failures.errors);
        let root = std::path::Path::new("./assets/packs/tw_json");
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(root).unwrap();
        pack.save_to_directory(root).expect("failed to save pack");
        Ok(())
    }
}
