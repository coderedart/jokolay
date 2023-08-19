use crate::prelude::*;
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};

use cap_std::fs::Dir;
use egui::Ui;
use egui_extras::{Column, TableRow};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use tracing::{field::Visit, Event, Level, Subscriber};
use tracing_subscriber::Layer;

struct JokolayTracingLayer;
static JKL_LOG_TRACING_BUFFER: OnceLock<Mutex<AllocRingBuffer<TracingEvent>>> = OnceLock::new();

pub fn install_tracing(jokolay_dir: &Dir) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};
    // get the log level
    let filter_layer = EnvFilter::try_from_env("JOKOLAY_LOG")
        .or_else(|_| EnvFilter::try_new("info,wgpu=warn,naga=warn"))
        .unwrap();
    // create log file in the data dir. This will also serve as a check that the directory is "writeable" by us
    let writer = std::io::BufWriter::new(
        jokolay_dir
            .create("jokolay.log")
            .into_diagnostic()
            .wrap_err("failed to create jokolay.log file")?,
    );
    let (nb, guard) = tracing_appender::non_blocking(writer);
    let fmt_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(nb);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(create_jokolay_tracing_layer())
        .init();
    Ok(guard)
}
/// code stolen from miette::set_panic_hook
pub fn install_miette_panic_hooks() -> Result<()> {
    miette::set_hook(Box::new(|diagnostic| {
        let handler = Box::new(miette::NarratableReportHandler::new());
        let mut panic_report = String::new();
        if let Err(e) = handler.render_report(&mut panic_report, diagnostic) {
            tracing::error!("failed to render report: {e}");
        }
        tracing::error!("crashing: {:#?}", &panic_report);
        handler
    }))
    .wrap_err("failed to install miette hook")?;

    #[derive(Debug, thiserror::Error, miette::Diagnostic)]
    #[error("{0}{}", Panic::backtrace())]
    #[diagnostic(help("set the `RUST_BACKTRACE=1` environment variable to display a backtrace."))]
    struct Panic(String);
    impl Panic {
        fn backtrace() -> String {
            use std::fmt::Write;
            if let Ok(var) = std::env::var("RUST_BACKTRACE") {
                if !var.is_empty() && var != "0" {
                    const HEX_WIDTH: usize = std::mem::size_of::<usize>() + 2;
                    // Padding for next lines after frame's address
                    const NEXT_SYMBOL_PADDING: usize = HEX_WIDTH + 6;
                    let mut backtrace = String::new();
                    let trace = backtrace::Backtrace::new();
                    let frames = backtrace_ext::short_frames_strict(&trace).enumerate();
                    for (idx, (frame, sub_frames)) in frames {
                        let ip = frame.ip();
                        let _ = write!(backtrace, "\n{:4}: {:2$?}", idx, ip, HEX_WIDTH);

                        let symbols = frame.symbols();
                        if symbols.is_empty() {
                            let _ = write!(backtrace, " - <unresolved>");
                            continue;
                        }

                        for (idx, symbol) in symbols[sub_frames].iter().enumerate() {
                            // Print symbols from this address,
                            // if there are several addresses
                            // we need to put it on next line
                            if idx != 0 {
                                let _ = write!(backtrace, "\n{:1$}", "", NEXT_SYMBOL_PADDING);
                            }

                            if let Some(name) = symbol.name() {
                                let _ = write!(backtrace, " - {}", name);
                            } else {
                                let _ = write!(backtrace, " - <unknown>");
                            }

                            // See if there is debug information with file name and line
                            if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                                let _ = write!(
                                    backtrace,
                                    "\n{:3$}at {}:{}",
                                    "",
                                    file.display(),
                                    line,
                                    NEXT_SYMBOL_PADDING
                                );
                            }
                        }
                    }
                    return backtrace;
                }
            }
            "".into()
        }
    }

    std::panic::set_hook(Box::new(|panic_info| {
        // code stolen from miette::set_panic_hook
        let mut message = "Something went wrong".to_string();
        let payload = panic_info.payload();
        if let Some(msg) = payload.downcast_ref::<&str>() {
            message = msg.to_string();
        }
        if let Some(msg) = payload.downcast_ref::<String>() {
            message = msg.clone();
        }
        let mut report: Result<()> = Err(Panic(message).into());
        if let Some(loc) = panic_info.location() {
            report = report
                .with_context(|| format!("at {}:{}:{}", loc.file(), loc.line(), loc.column()));
        }
        if let Err(err) = report.with_context(|| "Main thread panicked.".to_string()) {
            eprintln!("Error: {:?}", err);
            tracing::error!("crashing: {:?}", &err);
            // if let Err(e) = notify_rust::Notification::new()
            //     .appname("Jokolay")
            //     .body(&format!("{:?}", &err))
            //     .summary("Jokolay crashed")
            //     .timeout(0)
            //     .finalize()
            //     .show()
            // {
            //     tracing::error!("failed to display notification");
            //     eprintln!("failed to display notification, {e:?}");
            // }
        }
    }));
    Ok(())
}

#[derive(Debug)]
struct TracingEvent {
    level: Level,
    line: u32,
    target: String,
    message: String,
    #[allow(dead_code)]
    fields: BTreeMap<String, String>,
}

struct EventVisitor<'a>(&'a mut TracingEvent);
impl Visit for EventVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "message" => {
                self.0.message = format!("{value:?}");
            }
            "log.line" => {
                self.0.line = format!("{value:?}").parse().unwrap();
            }
            "log.target" => {
                self.0.target = format!("{value:?}");
            }
            _ => {
                if field.name().starts_with("log.") {
                    return;
                }
                let name = field.name().to_string();
                let value = format!("{value:?}");
                self.0.fields.insert(name, value);
            }
        }
    }
}
impl TracingEvent {
    fn from_event_and_ctx<S>(
        event: &Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> Self {
        let target = if event.metadata().target() == "log" {
            Default::default()
        } else {
            event.metadata().target().to_string()
        };
        let mut te = Self {
            level: event.metadata().level().clone(),
            line: event.metadata().line().unwrap_or_default(),
            target,
            message: Default::default(),
            fields: Default::default(),
        };
        event.record(&mut EventVisitor(&mut te));

        te
    }
    fn ui_row(&self, row: &mut TableRow) {
        row.col(|ui| {
            ui.label(format!("{}", self.level));
        });
        row.col(|ui| {
            ui.label(format!("{}", &self.target));
        });
        row.col(|ui| {
            ui.label(format!("{}", self.line));
        });
        row.col(|ui| {
            ui.label(format!("{}", &self.message));
        });
    }
}
impl<S: Subscriber> Layer<S> for JokolayTracingLayer {
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let te = TracingEvent::from_event_and_ctx(event, ctx);
        JKL_LOG_TRACING_BUFFER
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .push(te);
    }
}

pub fn create_jokolay_tracing_layer<S: Subscriber>() -> impl Layer<S> {
    assert!(JKL_LOG_TRACING_BUFFER
        .set(Mutex::new(AllocRingBuffer::new(128)))
        .is_ok());
    JokolayTracingLayer
}
pub fn show_tracing_events(ui: &mut Ui) {
    egui_extras::TableBuilder::new(ui)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(40.0))
        .column(Column::initial(100.0).range(40.0..=300.0).clip(true))
        .column(Column::exact(40.0))
        .column(Column::initial(100.0).clip(true))
        .min_scrolled_height(0.0)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("level");
            });
            header.col(|ui| {
                ui.strong("target");
            });
            header.col(|ui| {
                ui.strong("line");
            });
            header.col(|ui| {
                ui.strong("message");
            });
        })
        .body(|body| {
            let events = JKL_LOG_TRACING_BUFFER.get().unwrap().lock().unwrap();
            body.rows(20.0, events.len(), |index, mut row| {
                let ev = events.get(index as _).unwrap();
                ev.ui_row(&mut row);
            });
        });
}
