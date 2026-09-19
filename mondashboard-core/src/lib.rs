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

use thiserror::Error;

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
