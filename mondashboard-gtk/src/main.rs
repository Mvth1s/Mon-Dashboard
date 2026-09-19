// Les widgets ne sont pas encore instanciés tant que l'interface n'est pas
// câblée ; cette tolérance est retirée dès que app.rs les utilise.
#![allow(dead_code)]

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
