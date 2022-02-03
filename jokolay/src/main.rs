use jokolay::log_initialize;
use std::path::Path;

fn fake_main() -> anyhow::Result<()> {
    let _guard = log_initialize(
        Path::new("./assets"),
        tracing::level_filters::LevelFilter::INFO,
    )?;
    let mut window = jokolay::core::window::OverlayWindow::create([800, 600].into())?;
    let mut renderer = jokolay::core::renderer::Renderer::initialize_vulkan(&window, true)?;
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
        let input = window.tick()?;
        let (output, shapes) = etx.tick(input, &mut window)?;
        if etx.ctx.wants_pointer_input() || etx.ctx.wants_keyboard_input() {
            window.window.set_mouse_passthrough(false);
        } else {
            window.window.set_mouse_passthrough(false);

        }
        renderer.tick(output.textures_delta, shapes, &window)?;
    }
    Ok(())
}
fn main() {
    if let Err(e) = fake_main() {
        dbg!(e);
    }
}
