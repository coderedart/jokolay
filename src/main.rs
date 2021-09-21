use jokolay::JokolayApp;

fn main() -> anyhow::Result<()> {
    jokolay::log_init(
        log::LevelFilter::Info,
        log::LevelFilter::Trace,
        "./joko.log".into(),
    )?;
    let app = JokolayApp::new();
    log::trace!("app initialized.");
    app.run()?;
    Ok(())
}
