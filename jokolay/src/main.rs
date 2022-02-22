use anyhow::{bail, Context};
use jokolay::log_initialize;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use sysinfo::SystemExt;

use jokolay::config::ConfigManager;
use tracing::level_filters::LevelFilter;

fn fake_main() -> anyhow::Result<()> {
    let [config_dir, _data_dir, _cache_dir, _markers_dir, logs_dir, themes_dir, fonts_dir] =
        jokolay::get_config_data_cache_markers_dirs().map_err(|e| {
            rfd::MessageDialog::new()
                .set_title("failed to start jokolay")
                .set_description(&format!("failed to get current dir. error: {:#?}", &e))
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            e
        })?;

    let mut cm = match ConfigManager::new(config_dir.join("joko_config.json")) {
        Ok(cm) => cm,
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("failed to start jokolay")
                .set_description(&format!(
                    "failed to create config manager. error: {:#?}",
                    &e
                ))
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            anyhow::bail!(e)
        }
    };
    let log_level = match cm.config.log_level.as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        rest => {
            rfd::MessageDialog::new()
                .set_title("failed to parse log level")
                .set_description(&format!("failed to parse log level. source: {}", rest))
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            anyhow::bail!("log level wrong")
        }
    };
    let _guard = log_initialize(&logs_dir.join("jokolay.log"), log_level).map_err(|e| {
        rfd::MessageDialog::new()
            .set_title("failed to initiate logging")
            .set_description(&format!("log initialize failed error: {:#?}", &e))
            .set_level(rfd::MessageLevel::Error)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
        e
    })?;
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
    let mut window = jokolay::core::window::OverlayWindow::create(&cm.config)?;
    let mut renderer = handle.block_on(jokolay::core::renderer::Renderer::new(
        &window, &cm.config, true,
    ))?;
    let mut etx =
        jokolay::core::gui::Etx::new(&window, themes_dir, &cm.config.theme_name, fonts_dir)?;
    let window_id: u32 = match window.window.raw_window_handle() {
        RawWindowHandle::Xlib(x) => {
            #[cfg(target_os = "windows")]
            let xid = x.window;
            #[cfg(target_os = "linux")]
            let xid = x
                .window
                .try_into()
                .context("failed to convert raw handle into window_id")?;

            xid
        }
        RawWindowHandle::Xcb(x) => x.window,
        // RawWindowHandle::Wayland(x) => {x.surface.try_into().context("failed to convert raw handle into window_id")?}
        RawWindowHandle::Win32(_) | RawWindowHandle::WinRt(_) => 0,
        rest => bail!("invalid platform for raw window handle. {rest:#?}"),
    };
    let mut mm = jokolink::MumbleCtx::new(
        cm.config.mumble_config.clone(),
        window_id,
        window.glfw.get_time(),
    )?;
    let mut timer = std::time::Instant::now();
    let mut fps = 0u32;
    let mut sys = sysinfo::System::new();
    sys.refresh_all();
    while !window.window.should_close() {
        fps += 1;
        if timer.elapsed() > std::time::Duration::from_secs(1) {
            dbg!(fps);
            fps = 0;
            timer = std::time::Instant::now();
        }

        let input = window.tick(&mut renderer.wtx)?;
        let (output, shapes) = etx.tick(
            input,
            &mut window,
            &mut renderer.wtx,
            &mut cm,
            &mut mm,
            handle.clone(),
        )?;
        mm.tick(window.window_state.glfw_time, &mut sys)?;

        if etx.ctx.wants_pointer_input() || etx.ctx.wants_keyboard_input() {

            #[cfg(target_os = "linux")]
            if !window.window.is_mouse_passthrough() {
                // check if we have been clicked, while we are not passthrough but not focused either. it means mouse is being captured by gw2
                // and we will need to force focus to break that capture.
                if !window.window.is_mouse_passthrough() && ((!window.window_state.mouse_state[0].button_pressed[1] && window.window_state.mouse_state[1].button_pressed[1]) ||
                    (!window.window_state.mouse_state[0].button_pressed[2] && window.window_state.mouse_state[1].button_pressed[2]) ||
                        (!window.window_state.mouse_state[0].button_pressed[3] && window.window_state.mouse_state[1].button_pressed[3])) && !window.window.is_focused() {
                    window.window.focus();
                }

            }
            window.window.set_mouse_passthrough(false);
        } else {
            window.window.set_mouse_passthrough(true);
        }
        renderer.tick(output.textures_delta, shapes, &window)?;
    }
    tokio_quit_sender
        .send(())
        .context("failed to send tokio quit signal")?;
    if tokio_thread.join().is_err() {
        anyhow::bail!("failed to join tokio thread");
    }
    Ok(())
}

fn main() {
    if let Err(e) = fake_main() {
        dbg!(e);
    }
}
