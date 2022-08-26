use tracing::info;

fn main() -> color_eyre::Result<()> {
    {
        tracing_subscriber::fmt().init();
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
