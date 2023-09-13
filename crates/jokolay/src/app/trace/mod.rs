use std::sync::{Mutex, OnceLock};

use cap_std::fs_utf8::Dir;
use egui::Ui;
use egui_extras::{Column, TableRow};
use miette::{Context, IntoDiagnostic, Result};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use tracing::{field::Visit, span, Event, Level, Subscriber};
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
            .into_diagnostic()
            .wrap_err("failed to parse log filter levels from env")?;
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
    pub fn gui(etx: &egui::Context, open: &mut bool) {
        egui::Window::new("Tracing").open(open).show(etx, |ui| {
            Self::show_tracing_events(ui);
        });
    }
    fn show_tracing_events(ui: &mut Ui) {
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
}
impl Default for TracingEvent {
    fn default() -> Self {
        Self {
            level: Level::TRACE,
            target: Default::default(),
            message: Default::default(),
            notify: Default::default(),
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
            "log.target" => {
                self.0.target = format!("{value:?}");
            }
            _ => {}
        }
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.record_debug(field, &value)
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if field.name() == "notify" {
            self.0.notify = value as _;
        } else {
            self.record_debug(field, &value)
        }
    }
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if field.name() == "notify" {
            self.0.notify = value as _;
        } else {
            self.record_debug(field, &value)
        }
    }
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "notify" {
            self.0.notify = value as _;
        } else {
            self.record_debug(field, &value)
        }
    }
}
impl TracingEvent {
    fn ui_row(&self, row: &mut TableRow) {
        row.col(|ui| {
            ui.label(format!("{}", self.level));
        });
        row.col(|ui| {
            ui.label(self.target.to_string());
        });
        row.col(|ui| {
            ui.label(self.message.to_string());
        });
    }
}
impl<S: Subscriber> Layer<S> for JokolayTracingLayer {
    fn on_event(&self, event: &Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let level = *event.metadata().level();
        let sp = ctx
            .current_span()
            .metadata()
            .zip(ctx.current_span().id().cloned());
        let target = if event.metadata().target() == "log" {
            Default::default()
        } else {
            event.metadata().target().to_string()
        };
        let mut te = TracingEvent {
            level,
            target,
            notify: match level {
                Level::TRACE | Level::DEBUG | Level::INFO => 0.0,
                Level::WARN => 4.0,
                Level::ERROR => 7.0,
            },
            ..Default::default()
        };
        event.record(&mut EventVisitor(&mut te));

        let mut global_tracing_data = JKL_TRACING_DATA.get().unwrap().lock().unwrap();
        if te.notify > 0.1 {
            if let Some((md, id)) = sp {
                let message = te.message.clone();
                global_tracing_data
                    .notifications
                    .spans
                    .entry(id.clone())
                    .or_insert_with(|| SpanNotification {
                        title: md.name().to_string(),
                        latest_message: te.message.clone(),
                        level: te.level,
                        time_to_live: f32::MAX,
                    })
                    .latest_message = message;
            } else {
                global_tracing_data
                    .notifications
                    .current
                    .push(Notification {
                        title: te.target.clone(),
                        message: te.message.clone(),
                        level: te.level,
                        time_to_live: te.notify,
                    });
            }
        }
        global_tracing_data.buffer.push(te);
    }

    fn on_close(&self, id: span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(span_notif) = JKL_TRACING_DATA
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .notifications
            .spans
            .get_mut(&id)
        {
            span_notif.time_to_live = 3.0;
        }
    }
}

struct GlobalTracingData {
    pub buffer: AllocRingBuffer<TracingEvent>,
    pub notifications: Notifications,
}

#[derive(Debug, Default)]
struct Notifications {
    current: Vec<Notification>,
    spans: indexmap::IndexMap<span::Id, SpanNotification>,
}
impl Notifications {
    fn tick_egui(&mut self, etx: &egui::Context) {
        let dt = etx.input(|i| i.unstable_dt);
        egui::Window::new("Notifications")
            .anchor(egui::Align2::RIGHT_TOP, [0.0, 0.0])
            .interactable(true)
            .movable(false)
            .title_bar(false)
            .show(etx, |ui| {
                let persistent_notifs = std::mem::take(&mut self.spans);
                for (span_id, mut notif) in persistent_notifs {
                    // show notification
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&notif.title);
                            ui.add_space((ui.available_width() - 20.0).max(0.0));
                            if ui
                                .button(egui::RichText::new("X").color(egui::Color32::RED))
                                .clicked()
                            {
                                notif.time_to_live = 0.0;
                            }
                        });
                        ui.label(&notif.latest_message);
                    });
                    // reduce the ttl by the amount of time since last frame
                    notif.time_to_live -= dt;
                    // push to current if its still alive
                    if notif.time_to_live > 0.0 {
                        self.spans.insert(span_id, notif);
                    }
                }
                let notifs = std::mem::take(&mut self.current);
                for mut notif in notifs {
                    // show notification
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&notif.title);
                            ui.add_space((ui.available_width() - 20.0).max(0.0));
                            if ui
                                .button(egui::RichText::new("X").color(egui::Color32::RED))
                                .clicked()
                            {
                                notif.time_to_live = 0.0;
                            }
                        });
                        ui.label(&notif.message);
                    });
                    // reduce the ttl by the amount of time since last frame
                    notif.time_to_live -= dt;
                    // push to current if its still alive
                    if notif.time_to_live > 0.0 {
                        self.current.push(notif);
                    }
                }
            });
    }
}

#[derive(Debug)]
struct Notification {
    pub title: String,
    pub message: String,
    #[allow(unused)]
    pub level: Level,
    pub time_to_live: f32,
}
#[derive(Debug)]
struct SpanNotification {
    pub title: String,
    pub latest_message: String,
    #[allow(unused)]
    pub level: Level,
    pub time_to_live: f32,
}
