use anyhow::Context;
use jokolay::log_initialize;
use std::path::Path;

fn fake_main() -> anyhow::Result<()> {
    let _guard = log_initialize(
        Path::new("./assets"),
        tracing::level_filters::LevelFilter::INFO,
    )?;
    let (tokio_quit_sender, tokio_quit_receiver) = flume::bounded(1);
    let rt = tokio::runtime::Runtime::new()?;
    let handle = rt.handle().clone();
    let tokio_thread = std::thread::spawn(move || {
        rt.block_on(async {
            tokio_quit_receiver
                .recv_async()
                .await
                .expect("failed to receive tokio quit signal")
        });
    });
    let mut window = jokolay::core::window::OverlayWindow::create([800, 600].into())?;
    let mut renderer = handle.block_on(jokolay::core::renderer::Renderer::new(&window, true))?;
    let mut etx = jokolay::core::gui::Etx::new(&window)?;
    let mut timer = std::time::Instant::now();
    let mut fps = 0u32;

    while !window.window.should_close() {
        fps += 1;
        if timer.elapsed() > std::time::Duration::from_secs(1) {
            dbg!(fps);
            fps = 0;
            timer = std::time::Instant::now();
        }
        let input = window.tick(&mut renderer.wtx)?;
        let (output, shapes) = etx.tick(input, &mut window)?;
        if etx.ctx.wants_pointer_input() || etx.ctx.wants_keyboard_input() {
            window.window.set_mouse_passthrough(false);
        } else {
            window.window.set_mouse_passthrough(true);
        }
        renderer.tick(output.textures_delta, shapes, &window)?;
    }
    tokio_quit_sender
        .send(())
        .context("failed to send tokio quit signal")?;
    tokio_thread.join().expect("failed to join tokio thread");
    Ok(())
}
fn main() {
    if let Err(e) = fake_main() {
        dbg!(e);
    }
}
