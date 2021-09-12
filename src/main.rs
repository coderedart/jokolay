use jokolay::JokolayApp;

fn main() -> anyhow::Result<()> {
    jokolay::log_init(
        log::LevelFilter::Error,
        log::LevelFilter::Trace,
        "./joko.log".into(),
    )?;
    let app = JokolayApp::new();
    log::trace!("app initialized.");
    app.run()?;
    Ok(())
}
