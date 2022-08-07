use bevy::prelude::App;
use jokolay_desktop::add_desktop_addons;

fn main() {
    let mut app = App::new();

    add_desktop_addons(&mut app);
    app.run();
}
