mod app;
mod layout;
mod overlay;
mod polling;
mod settings;
mod tray;
mod widgets;

fn main() {
    env_logger::init();
    app::run();
}
