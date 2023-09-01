mod notification;

use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};

use cap_std::fs_utf8::Dir;
use egui::Ui;
use egui_extras::{Column, TableRow};
use miette::{Context, IntoDiagnostic, Result};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use tracing::{field::Visit, Event, Level, Subscriber};
use tracing_subscriber::Layer;
pub struct JokolayTracingLayer;
static JKL_TRACING_DATA: OnceLock<Mutex<GlobalTracingData>> = OnceLock::new();

impl JokolayTracingLayer {
    pub fn install_tracing(
        jokolay_dir: &Dir,
    ) -> Result<tracing_appender::non_blocking::WorkerGuard> {
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
            .with_target(false)
            .pretty()
            .with_file(true)
            .with_line_number(true)
            .with_writer(nb);
        assert!(JKL_TRACING_DATA
            .set(Mutex::new(GlobalTracingData {
                buffer: AllocRingBuffer::new(128),
                notifications: Default::default()
            }))
            .is_ok());

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(JokolayTracingLayer)
            .init();
        Ok(guard)
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
                let events = &JKL_TRACING_DATA.get().unwrap().lock().unwrap().buffer;
                body.rows(20.0, events.len(), |index, mut row| {
                    let ev = events.get(index as _).unwrap();
                    ev.ui_row(&mut row);
                });
            });
    }
    pub fn show_notifications(etx: &egui::Context) {
        JKL_TRACING_DATA
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .notifications
            .tick_egui(etx);
    }
}

/// A tracing even that we store in our Joko Tracing storage
/// The main purpose is to use this as a notification system.
/// When we get a warn/error log, we will show it as a notification.
/// This will allow us to just dump errors into the tracing infrastructure and automatically alert the user about the error.
#[derive(Debug)]
struct TracingEvent {
    /// Level of the event
    level: Level,
    /// the line in source where this event was triggered
    line: u32,
    /// The target of the event. Usually the module/function scope at which the even was triggered
    /// In future, we can use this as the "identifier" of a particular group/class of events which can be special cased
    /// eg: we can say that if the target string starts with "LUA", we will consider this an error from a plugin
    target: String,
    /// The actual message of the event. usually the formatted string from the warn/error macros.
    /// It is recommended to keep those strings static and instead record any variables as "fields" to give us more flexibility with regards to formatting
    message: String,
    /// This is the length for which we will show the notification
    /// This is recorded as a field from the event. So, make sure to set it to 0u64 if you don't want to display the log as a notification.
    /// the value must be u64 and in seconds.
    notify: f32,
    /// These are the fields recorded from this event
    /// We can eventually optimize this as an enum to avoid allocating for primitives like bool/numbers.
    fields: BTreeMap<String, String>,
}
impl Default for TracingEvent {
    fn default() -> Self {
        Self {
            level: Level::TRACE,
            line: Default::default(),
            target: Default::default(),
            message: Default::default(),
            notify: Default::default(),
            fields: Default::default(),
        }
    }
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

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "notify" {
            self.0.notify = value as _;
        } else {
            self.record_debug(field, &value)
        }
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.record_debug(field, &value)
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
        let level = *event.metadata().level();
        let mut te = Self {
            level,
            line: event.metadata().line().unwrap_or_default(),
            target,
            notify: if level < Level::INFO { 10.0 } else { 0.0 },
            ..Default::default()
        };
        event.record(&mut EventVisitor(&mut te));

        te
    }
    fn ui_row(&self, row: &mut TableRow) {
        row.col(|ui| {
            ui.label(format!("{}", self.level));
        });
        row.col(|ui| {
            ui.label(self.target.to_string());
        });
        row.col(|ui| {
            ui.label(format!("{}", self.line));
        });
        row.col(|ui| {
            ui.label(self.message.to_string());
        });
    }
}
impl<S: Subscriber> Layer<S> for JokolayTracingLayer {
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let te = TracingEvent::from_event_and_ctx(event, ctx);

        JKL_TRACING_DATA
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .add_event(te);
    }
}

struct GlobalTracingData {
    pub buffer: AllocRingBuffer<TracingEvent>,
    pub notifications: notification::Notifications,
}
impl GlobalTracingData {
    pub fn add_event(&mut self, ev: TracingEvent) {
        self.notifications.add_event(&ev);
        self.buffer.push(ev);
    }
}
