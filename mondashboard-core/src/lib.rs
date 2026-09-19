pub mod alerts;
pub mod battery;
pub mod config;
pub mod cpu;
pub mod disk;
pub mod fans;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod process;
pub mod sysfs;

use thiserror::Error;

/// Vrai lorsque l'application tourne dans un bac à sable Flatpak.
///
/// L'isolement y change ce que le système donne à voir : les points de
/// montage sont ceux du bac à sable, et la liste des processus se limite à
/// ceux de l'application. Les widgets concernés doivent le dire plutôt que
/// d'afficher une information trompeuse.
pub fn is_sandboxed() -> bool {
    std::env::var_os("FLATPAK_ID").is_some() || std::path::Path::new("/.flatpak-info").exists()
}

#[derive(Debug, Error)]
pub enum MonDashboardError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("D-Bus error: {0}")]
    DBus(String),
    #[error("GPU error: {0}")]
    Gpu(String),
    #[error("Process error: {0}")]
    Process(String),
}
