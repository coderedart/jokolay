use jokolay::JokolayApp;

fn main() -> anyhow::Result<()> {
    jokolay::log_init(
        log::LevelFilter::Error,
        log::LevelFilter::Warn,
        "./joko.log".into(),
    )?;
    let app = JokolayApp::new()?;
    app.run()?;
    Ok(())
}
