use tracing::Level;

use super::TracingEvent;

#[derive(Debug, Default)]
pub struct Notifications {
    current: Vec<Notification>,
}
impl Notifications {
    pub fn tick_egui(&mut self, etx: &egui::Context) {
        let dt = etx.input(|i| i.unstable_dt);
        egui::Area::new("Notifications")
            .anchor(egui::Align2::RIGHT_TOP, [0.0, 0.0])
            .interactable(true)
            .movable(false)
            .show(etx, |ui| {
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
                    notif.time_to_live = notif.time_to_live - dt;
                    // push to current if its still alive
                    if notif.time_to_live > 0.0 {
                        self.current.push(notif);
                    }
                }
            });
    }
    pub(super) fn add_event(&mut self, ev: &TracingEvent) {
        if ev.level < Level::INFO {
            self.current.push(Notification {
                title: ev.target.clone(),
                message: ev.message.clone(),
                level: ev.level,
                time_to_live: ev.notify,
            });
        }
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
