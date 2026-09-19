use serde::{Deserialize, Serialize};

use crate::alerts::AlertConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl Default for TemperatureUnit {
    fn default() -> Self {
        Self::Celsius
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverlayConfig {
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_vram: bool,
    pub show_ram: bool,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetSpecificConfig {
    Cpu { show_per_core: bool },
    Gpu { selected_gpu_index: Option<usize> },
    Network { interface: Option<String>, ping_host: Option<String> },
    Process { sort_by_cpu: bool, max_rows: u32 },
    Fans { hidden_labels: Vec<String> },
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPlacement {
    /// e.g. "cpu", "gpu_0", "disk_sda"
    pub widget_id: String,
    pub col: u32,
    pub row: u32,
    pub col_span: u32,
    pub row_span: u32,
    pub visible: bool,
    pub widget_config: WidgetSpecificConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub name: String,
    pub placements: Vec<WidgetPlacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Bumped on breaking schema changes for migration.
    pub version: u32,
    pub refresh_interval_secs: u32,
    pub temperature_unit: TemperatureUnit,
    pub ping_host: String,
    pub alerts: AlertConfig,
    pub layouts: Vec<LayoutConfig>,
    pub active_layout: String,
    pub overlay: OverlayConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            refresh_interval_secs: 2,
            temperature_unit: TemperatureUnit::Celsius,
            ping_host: "1.1.1.1".to_string(),
            alerts: AlertConfig::default(),
            layouts: vec![],
            active_layout: "Default".to_string(),
            overlay: OverlayConfig::default(),
        }
    }
}
