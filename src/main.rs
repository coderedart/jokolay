use jokolay::JokolayApp;

fn main() -> anyhow::Result<()> {
    jokolay::log_init(
        log::LevelFilter::Error,
        log::LevelFilter::Trace,
        "./joko.log".into(),
    )?;
    let mut app = JokolayApp::new()?;
    app.run()?;
    Ok(())
}
