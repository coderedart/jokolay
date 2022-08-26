use egui_overlay::start_overlay;

fn main() {
    use jokolay::Jokolay;

    let jokolay = Jokolay::default();
    start_overlay(jokolay);
}
